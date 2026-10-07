import { create } from "zustand";
import { RemoteApi, RemoteApiError } from "./api";
import { importPairingKey, parsePairingUrl } from "./crypto";
import { clearCredential, loadCredential, saveCredential } from "./persistence";
import { applyEvent } from "./protocol";
import type {
  CredentialState,
  Host,
  Project,
  SessionSnapshot,
  SessionSummary,
  SseEnvelope,
} from "./types";

export type StreamState = "idle" | "connecting" | "live" | "reconnecting" | "failed";

interface WebStore {
  /**
   * The paired Host's credential, including its endpoint. It is the single source of truth for
   * the pairing; `credential.host` is refreshed from the Host detail on every remote load.
   */
  credential: CredentialState | null;
  projects: Project[];
  sessions: SessionSummary[];
  selectedSessionId: string | null;
  snapshot: SessionSnapshot | null;
  pairingError: string | null;
  error: string | null;
  loading: boolean;
  streamState: StreamState;
  lastEventId: string | null;
  reviewOpen: boolean;
  hydrated: boolean;
  hydrate: () => Promise<void>;
  selectSession: (sessionId: string) => Promise<void>;
  toggleReview: () => void;
  /** Resolves true when pairing succeeded; failures are reported through `pairingError`. */
  pairFromUrl: (value: string, deviceName?: string) => Promise<boolean>;
  /** Resolves true when the message was accepted; failures are reported through `error`. */
  sendMessage: (text: string) => Promise<boolean>;
  resolveApproval: (action: "allow_once" | "allow_session" | "deny") => Promise<void>;
  replyQuestion: (answer: string) => Promise<void>;
  cancelTurn: () => Promise<void>;
  retryTurn: () => Promise<void>;
  unpair: () => Promise<void>;
  /** Opens the selected Session's stream and returns its cleanup. */
  connectStream: () => () => void;
  setEncryptionEnabled: (enabled: boolean) => void;
}

/** The paired Host, or null when this browser is not paired. */
export const selectHost = (state: WebStore): Host | null => state.credential?.host ?? null;

const errorMessage = (error: unknown, fallback = "Remote request failed.") =>
  error instanceof Error ? error.message : fallback;

export const useWebStore = create<WebStore>((set, get) => {
  let streamCleanup: (() => void) | undefined;
  let streamGeneration = 0;
  const disconnectStream = () => {
    streamGeneration += 1;
    const cleanup = streamCleanup;
    streamCleanup = undefined;
    cleanup?.();
  };
  const updateCredential = (credential: CredentialState | null) => {
    saveCredential(credential);
    set({ credential });
  };
  const api = new RemoteApi(() => get().credential, updateCredential);
  const setError = (error: unknown) => set({ error: errorMessage(error) });

  /**
   * Runs a Session action when this browser is paired and a Session is selected. Error policy:
   * a failure is reported through `error` and is not rethrown; the result is false instead.
   */
  const withSession = async (
    fn: (sessionId: string, snapshot: SessionSnapshot | null) => Promise<void>,
  ): Promise<boolean> => {
    const { credential, selectedSessionId, snapshot } = get();
    if (!credential || !selectedSessionId) return false;
    try {
      await fn(selectedSessionId, snapshot);
      return true;
    } catch (error) {
      setError(error);
      return false;
    }
  };

  /** Loads a Session snapshot. Returns false (with `error` set) when it could not be loaded. */
  const loadSnapshot = async (sessionId: string): Promise<boolean> => {
    set({
      loading: true,
      error: null,
      selectedSessionId: sessionId,
      snapshot: null,
      lastEventId: null,
      streamState: "connecting",
    });
    try {
      const project = get().sessions.find((session) => session.id === sessionId)?.project;
      const snapshot = await api.getSession(sessionId, project);
      // The REST snapshot exposes eventSequence but not an SSE event ID, and event IDs are opaque,
      // so the stream opens without Last-Event-ID. The server then sends an atomic session.snapshot
      // first, which closes any gap between this fetch and stream registration.
      set({ snapshot, loading: false, lastEventId: null });
      return true;
    } catch (error) {
      set({ loading: false, streamState: "failed" });
      setError(error);
      return false;
    }
  };

  /** Loads the Host, Projects, and Sessions, then opens the first Session. Throws on failure. */
  const loadRemote = async (credential: CredentialState) => {
    try {
      updateCredential(credential);
      const [host, projectData, sessionData] = await Promise.all([
        api.getHost(),
        api.listProjects(),
        api.listSessions(),
      ]);
      // Read back the credential: a token refresh during the load may have replaced it.
      const current = get().credential ?? credential;
      updateCredential({ ...current, host });
      set({
        projects: projectData.items,
        sessions: sessionData.items,
        selectedSessionId: null,
        snapshot: null,
        loading: false,
        hydrated: true,
        streamState: "idle",
        error: null,
      });
      if (sessionData.items[0]) await get().selectSession(sessionData.items[0].id);
    } catch (error) {
      set({ loading: false, streamState: "failed" });
      setError(error);
      throw error;
    }
  };

  return {
    credential: null,
    projects: [],
    sessions: [],
    selectedSessionId: null,
    snapshot: null,
    pairingError: null,
    error: null,
    loading: false,
    streamState: "idle",
    lastEventId: null,
    reviewOpen: false,
    hydrated: false,
    hydrate: async () => {
      const saved = loadCredential();
      if (!saved) {
        set({ hydrated: true });
        return;
      }
      try {
        if (!saved.endpoint) throw new Error("Saved pairing endpoint is missing.");
        const credential: CredentialState = {
          ...saved,
          e2eKey: saved.e2eKeyRaw ? await importPairingKey(saved.e2eKeyRaw) : undefined,
          encryptionEnabled: saved.encryptionEnabled ?? Boolean(saved.e2eKeyRaw),
        };
        set({ credential, hydrated: true });
        await loadRemote(credential);
      } catch (error) {
        clearCredential();
        set({
          hydrated: true,
          credential: null,
          error: errorMessage(error, "Saved pairing could not be restored."),
        });
      }
    },
    selectSession: async (sessionId) => {
      const { credential, selectedSessionId, snapshot } = get();
      if (!credential || (sessionId === selectedSessionId && snapshot)) return;
      disconnectStream();
      await loadSnapshot(sessionId);
    },
    toggleReview: () => set((state) => ({ reviewOpen: !state.reviewOpen })),
    setEncryptionEnabled: (enabled) => {
      const credential = get().credential;
      if (!credential) return;
      updateCredential({
        ...credential,
        encryptionEnabled: enabled && Boolean(credential.e2eKey),
      });
    },
    pairFromUrl: async (value, deviceName = "SunCode Web") => {
      try {
        const pairing = parsePairingUrl(value);
        const result = await api.exchangePairing(pairing.endpoint, pairing, deviceName);
        const e2eKey = pairing.key ? await importPairingKey(pairing.key) : undefined;
        const credential: CredentialState = {
          ...result,
          endpoint: pairing.endpoint,
          e2eKey,
          e2eKeyRaw: pairing.key,
          encryptionEnabled: pairing.e2e && Boolean(e2eKey),
        };
        updateCredential(credential);
        set({ pairingError: null, error: null });
        await loadRemote(credential);
        return true;
      } catch (error) {
        const message = errorMessage(error, "Pairing failed.");
        set({ pairingError: message, error: message });
        return false;
      }
    },
    sendMessage: async (text) => {
      const trimmed = text.trim();
      if (!trimmed) return false;
      const markState = (sessionId: string, from: string | null, to: SessionSnapshot["state"]) =>
        set((state) => ({
          snapshot:
            state.snapshot?.id === sessionId && (from === null || state.snapshot.state === from)
              ? { ...state.snapshot, state: to }
              : state.snapshot,
          sessions: state.sessions.map((session) =>
            session.id === sessionId && (from === null || session.state === from)
              ? { ...session, state: to }
              : session,
          ),
        }));
      const { credential, selectedSessionId, snapshot } = get();
      if (!credential || !selectedSessionId) return false;
      const previous = snapshot?.state ?? "idle";
      markState(selectedSessionId, null, "running");
      const sent = await withSession((sessionId) => api.sendMessage(sessionId, trimmed));
      // Roll back the optimistic running state unless the stream already moved it on.
      if (sent) set({ error: null });
      else markState(selectedSessionId, "running", previous);
      return sent;
    },
    resolveApproval: async (action) => {
      await withSession(async (sessionId, snapshot) => {
        const approval = snapshot?.pendingApproval;
        if (!approval) return;
        await api.resolveApproval(sessionId, approval.id, action, approval.revision);
      });
    },
    replyQuestion: async (answer) => {
      await withSession(async (sessionId, snapshot) => {
        const question = snapshot?.pendingQuestion;
        if (!question || !answer.trim()) return;
        await api.replyQuestion(sessionId, question.id, [answer.trim()], question.revision);
      });
    },
    cancelTurn: async () => {
      await withSession((sessionId) => api.cancel(sessionId));
    },
    retryTurn: async () => {
      await withSession((sessionId) => api.retry(sessionId));
    },
    unpair: async () => {
      const { credential } = get();
      disconnectStream();
      if (credential) {
        try {
          await api.logout();
        } catch {
          /* local unpair still clears browser credentials */
        }
      }
      clearCredential();
      set({
        projects: [],
        sessions: [],
        selectedSessionId: null,
        snapshot: null,
        credential: null,
        error: null,
        streamState: "idle",
      });
    },
    connectStream: () => {
      const { credential, selectedSessionId, snapshot } = get();
      if (!credential || !selectedSessionId || !snapshot || snapshot.id !== selectedSessionId)
        return () => undefined;
      disconnectStream();
      const generation = streamGeneration;
      set({ streamState: "connecting" });
      const onEvent = (event: SseEnvelope) => {
        if (generation !== streamGeneration) return;
        if (event.session_id && event.session_id !== get().selectedSessionId) return;
        set((state) => {
          const current = state.snapshot;
          // Drop replayed events already represented by the snapshot; snapshots always apply.
          if (current && !event.snapshot && event.sequence <= current.eventSequence) {
            return { lastEventId: event.event_id };
          }
          const next = current
            ? applyEvent(current, event)
            : event.snapshot
              ? {
                  ...event.snapshot,
                  project:
                    state.sessions.find((session) => session.id === selectedSessionId)?.project ??
                    event.snapshot.project,
                }
              : null;
          return {
            lastEventId: event.event_id,
            snapshot: next,
            sessions:
              current && next
                ? state.sessions.map((session) =>
                    session.id === selectedSessionId
                      ? {
                          ...session,
                          state: next.state,
                          updatedAt: next.updatedAt,
                          preview: next.preview,
                        }
                      : session,
                  )
                : state.sessions,
          };
        });
      };
      const onError = async (error: unknown) => {
        if (generation !== streamGeneration) return;
        if (error instanceof RemoteApiError && error.status === 410) {
          // The cursor expired: reload a fresh snapshot and reopen the stream from it.
          if (await loadSnapshot(selectedSessionId)) {
            if (generation === streamGeneration) streamCleanup = get().connectStream();
            return;
          }
        }
        // streamSession retries transient failures itself; anything reaching here is terminal.
        set({ streamState: "failed" });
        setError(error);
      };
      const cleanup = api.streamSession(selectedSessionId, get().lastEventId ?? undefined, {
        onEvent,
        onError: (error) => void onError(error),
        onStateChange: (connection) => {
          if (generation !== streamGeneration) return;
          set({ streamState: connection });
        },
      });
      let disposed = false;
      const guardedCleanup = () => {
        if (disposed) return;
        disposed = true;
        if (streamCleanup === guardedCleanup) {
          streamCleanup = undefined;
          streamGeneration += 1;
        }
        cleanup();
      };
      streamCleanup = guardedCleanup;
      return guardedCleanup;
    },
  };
});
