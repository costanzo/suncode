import { create } from "zustand";
import { RemoteApi, RemoteApiError } from "./api";
import { importPairingKey, parsePairingUrl } from "./crypto";
import type {
  CredentialState,
  Host,
  PairingPayload,
  Project,
  SessionMessage,
  SessionSnapshot,
  SessionState,
  SessionSummary,
  SseEnvelope,
} from "./types";

const STORAGE_KEY = "suncode.web.credentials.v1";

type PersistedCredential = Omit<CredentialState, "e2eKey"> & { e2eKeyRaw?: string };
function persist(credential: CredentialState | null) {
  if (!credential) {
    sessionStorage.removeItem(STORAGE_KEY);
    return;
  }
  const { e2eKey, ...serializable } = credential;
  void e2eKey;
  sessionStorage.setItem(STORAGE_KEY, JSON.stringify(serializable));
}
function readPersisted(): PersistedCredential | null {
  try {
    const value = sessionStorage.getItem(STORAGE_KEY);
    return value ? (JSON.parse(value) as PersistedCredential) : null;
  } catch {
    return null;
  }
}

function eventText(value: unknown): string | null {
  if (typeof value === "string") return value;
  if (!value || typeof value !== "object") return null;
  const object = value as Record<string, unknown>;
  if (typeof object.text === "string") return object.text;
  const content = object.content;
  if (Array.isArray(content))
    return content
      .map((part) => eventText(part))
      .filter(Boolean)
      .join("");
  if (content && typeof content === "object") return eventText(content);
  return null;
}

function payloadRecord(payload: Record<string, unknown> | undefined) {
  return payload ?? {};
}
function nextSessionState(
  type: string,
  payload: Record<string, unknown>,
  current: SessionState,
): SessionState {
  if (type === "approval.requested") return "waiting_for_approval";
  if (type === "question.asked") return "waiting_for_answer";
  if (
    type === "turn.completed" ||
    type === "approval.resolved" ||
    type === "question.replied" ||
    type === "question.rejected"
  )
    return "idle";
  if (type === "provider.exchange.failed" || type === "turn.failed") return "failed";
  if (type === "turn.state") {
    const state = String(payload.state ?? "");
    if (state.includes("fail")) return "failed";
    if (state.includes("wait"))
      return state.includes("approval") ? "waiting_for_approval" : "waiting_for_answer";
    if (state.includes("run") || state.includes("queue")) return "running";
    if (state.includes("complete") || state === "idle") return "idle";
  }
  return current;
}

function applyEvent(snapshot: SessionSnapshot, event: SseEnvelope): SessionSnapshot {
  if (event.snapshot) return event.snapshot;
  const type = event.event_type ?? "";
  const payload = payloadRecord(event.payload);
  let messages = snapshot.messages;
  const messageValue = payload.message as Record<string, unknown> | undefined;
  const text = eventText(messageValue ?? payload.text);
  const messageId = String(payload.message_id ?? payload.id ?? `event-${event.sequence}`);
  const role =
    type === "message.user" ? "user" : type === "message.tool" ? "thinking" : "assistant";
  if (
    (type === "message.user" || type === "message.assistant" || type === "message.tool") &&
    text
  ) {
    const next: SessionMessage = {
      id: messageId,
      role,
      text,
      createdAt: String(payload.occurred_at ?? event.occurred_at ?? new Date().toISOString()),
    };
    messages = messages.some((item) => item.id === next.id)
      ? messages.map((item) => (item.id === next.id ? next : item))
      : [...messages, next];
  } else if (type === "assistant.delta" && text) {
    const last = messages[messages.length - 1];
    messages =
      last?.role === "assistant"
        ? [...messages.slice(0, -1), { ...last, text: `${last.text}${text}` }]
        : [
            ...messages,
            { id: messageId, role: "assistant", text, createdAt: new Date().toISOString() },
          ];
  }
  let pendingApproval = snapshot.pendingApproval;
  let pendingQuestion = snapshot.pendingQuestion;
  if (type === "approval.requested") {
    const approval = (payload.approval ?? payload) as Record<string, unknown>;
    pendingApproval = {
      id: String(approval.id ?? payload.approval_id ?? messageId),
      revision: Number(approval.revision ?? event.session_revision),
      summary: String(approval.summary ?? "Approval required"),
      scope: String(approval.scope ?? approval.detail ?? ""),
    };
  } else if (type === "approval.resolved") pendingApproval = null;
  if (type === "question.asked") {
    const question = (payload.question ?? payload) as Record<string, unknown>;
    pendingQuestion = {
      id: String(question.id ?? payload.question_id ?? messageId),
      revision: Number(question.revision ?? event.session_revision),
      prompt: String(question.prompt ?? "SunCode needs an answer"),
      options: Array.isArray(question.options) ? question.options.map(String) : [],
    };
  } else if (type === "question.replied" || type === "question.rejected") pendingQuestion = null;
  return {
    ...snapshot,
    revision: Math.max(snapshot.revision, event.session_revision),
    eventSequence: Math.max(snapshot.eventSequence, event.sequence),
    state: nextSessionState(type, payload, snapshot.state),
    updatedAt: event.occurred_at ?? new Date().toISOString(),
    messages,
    pendingApproval,
    pendingQuestion,
    preview: text ?? snapshot.preview,
  };
}

interface WebStore {
  endpoint: string;
  host: Host | null;
  projects: Project[];
  sessions: SessionSummary[];
  selectedSessionId: string | null;
  snapshot: SessionSnapshot | null;
  credential: CredentialState | null;
  pairing: PairingPayload | null;
  pairingError: string | null;
  error: string | null;
  loading: boolean;
  streamState: "idle" | "connecting" | "live" | "reconnecting" | "failed";
  lastEventId: string | null;
  reviewOpen: boolean;
  hydrated: boolean;
  api: RemoteApi;
  hydrate: () => Promise<void>;
  selectSession: (sessionId: string) => Promise<void>;
  toggleReview: () => void;
  pairFromUrl: (value: string, deviceName?: string) => Promise<void>;
  loadRemote: (endpoint: string, credentials: CredentialState) => Promise<void>;
  sendMessage: (text: string) => Promise<void>;
  resolveApproval: (action: "allow_once" | "allow_session" | "deny") => Promise<void>;
  replyQuestion: (answer: string) => Promise<void>;
  cancelTurn: () => Promise<void>;
  retryTurn: () => Promise<void>;
  unpair: () => Promise<void>;
  connectStream: () => () => void;
}

export const useWebStore = create<WebStore>((set, get) => {
  let streamCleanup: (() => void) | undefined;
  const updateCredential = (credential: CredentialState | null) => {
    persist(credential);
    set({ credential });
  };
  const api = new RemoteApi(() => get().credential, updateCredential);
  const setError = (error: unknown) =>
    set({ error: error instanceof Error ? error.message : "Remote request failed." });
  const loadSnapshot = async (sessionId: string, endpoint: string, credential: CredentialState) => {
    set({
      loading: true,
      error: null,
      selectedSessionId: sessionId,
      snapshot: null,
      lastEventId: null,
      streamState: "connecting",
    });
    try {
      const snapshot = await api.getSession(endpoint, credential.host.id, sessionId);
      set({ snapshot, loading: false, lastEventId: null });
    } catch (error) {
      set({ loading: false, streamState: "failed" });
      setError(error);
      throw error;
    }
  };
  return {
    endpoint: "",
    host: null,
    projects: [],
    sessions: [],
    selectedSessionId: null,
    snapshot: null,
    credential: null,
    pairing: null,
    pairingError: null,
    error: null,
    loading: false,
    streamState: "idle",
    lastEventId: null,
    reviewOpen: false,
    hydrated: false,
    api,
    hydrate: async () => {
      const saved = readPersisted();
      if (!saved) {
        set({ hydrated: true });
        return;
      }
      try {
        const credential: CredentialState = {
          ...saved,
          e2eKey: saved.e2eKeyRaw ? await importPairingKey(saved.e2eKeyRaw) : undefined,
        };
        const endpoint = sessionStorage.getItem(`${STORAGE_KEY}.endpoint`) ?? "";
        if (!endpoint) throw new Error("Saved pairing endpoint is missing.");
        set({ credential, endpoint, host: credential.host, hydrated: true });
        await get().loadRemote(endpoint, credential);
      } catch (error) {
        sessionStorage.removeItem(STORAGE_KEY);
        sessionStorage.removeItem(`${STORAGE_KEY}.endpoint`);
        set({
          hydrated: true,
          credential: null,
          host: null,
          error: error instanceof Error ? error.message : "Saved pairing could not be restored.",
        });
      }
    },
    selectSession: async (sessionId) => {
      const { endpoint, credential } = get();
      if (!credential || !endpoint || (sessionId === get().selectedSessionId && get().snapshot))
        return;
      streamCleanup?.();
      streamCleanup = undefined;
      try {
        await loadSnapshot(sessionId, endpoint, credential);
      } catch {
        /* error is rendered by the store */
      }
    },
    toggleReview: () => set((state) => ({ reviewOpen: !state.reviewOpen })),
    pairFromUrl: async (value, deviceName = "SunCode Web") => {
      try {
        const pairing = parsePairingUrl(value);
        const result = await api.exchangePairing(pairing.endpoint, pairing, deviceName);
        const e2eKey = pairing.key ? await importPairingKey(pairing.key) : undefined;
        const credential: CredentialState = { ...result, e2eKey, e2eKeyRaw: pairing.key };
        sessionStorage.setItem(`${STORAGE_KEY}.endpoint`, pairing.endpoint);
        updateCredential(credential);
        set({ pairing, pairingError: null, error: null });
        await get().loadRemote(pairing.endpoint, credential);
      } catch (error) {
        const message = error instanceof Error ? error.message : "Pairing failed.";
        set({ pairingError: message, error: message });
        throw error;
      }
    },
    loadRemote: async (endpoint, credential) => {
      try {
        updateCredential(credential);
        const [host, projectData, sessionData] = await Promise.all([
          api.getHost(endpoint, credential.host.id),
          api.listProjects(endpoint, credential.host.id),
          api.listSessions(endpoint, credential.host.id),
        ]);
        set({
          endpoint,
          credential,
          host,
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
    },
    sendMessage: async (text) => {
      const { endpoint, credential, selectedSessionId } = get();
      if (!credential || !endpoint || !selectedSessionId || !text.trim()) return;
      try {
        await api.sendMessage(endpoint, credential.host.id, selectedSessionId, text.trim());
        set({ error: null });
      } catch (error) {
        setError(error);
        throw error;
      }
    },
    resolveApproval: async (action) => {
      const { endpoint, credential, selectedSessionId, snapshot } = get();
      if (!credential || !endpoint || !selectedSessionId || !snapshot?.pendingApproval) return;
      try {
        await api.resolveApproval(
          endpoint,
          credential.host.id,
          selectedSessionId,
          snapshot.pendingApproval.id,
          action,
          snapshot.pendingApproval.revision,
        );
      } catch (error) {
        setError(error);
        throw error;
      }
    },
    replyQuestion: async (answer) => {
      const { endpoint, credential, selectedSessionId, snapshot } = get();
      if (
        !credential ||
        !endpoint ||
        !selectedSessionId ||
        !snapshot?.pendingQuestion ||
        !answer.trim()
      )
        return;
      try {
        await api.replyQuestion(
          endpoint,
          credential.host.id,
          selectedSessionId,
          snapshot.pendingQuestion.id,
          [answer.trim()],
          snapshot.pendingQuestion.revision,
        );
      } catch (error) {
        setError(error);
        throw error;
      }
    },
    cancelTurn: async () => {
      const { endpoint, credential, selectedSessionId } = get();
      if (!credential || !endpoint || !selectedSessionId) return;
      try {
        await api.cancel(endpoint, credential.host.id, selectedSessionId);
      } catch (error) {
        setError(error);
        throw error;
      }
    },
    retryTurn: async () => {
      const { endpoint, credential, selectedSessionId } = get();
      if (!credential || !endpoint || !selectedSessionId) return;
      try {
        await api.retry(endpoint, credential.host.id, selectedSessionId);
      } catch (error) {
        setError(error);
        throw error;
      }
    },
    unpair: async () => {
      const { endpoint, credential } = get();
      streamCleanup?.();
      streamCleanup = undefined;
      if (endpoint && credential) {
        try {
          await api.logout(endpoint, credential.host.id);
        } catch {
          /* local unpair still clears browser credentials */
        }
      }
      sessionStorage.removeItem(STORAGE_KEY);
      sessionStorage.removeItem(`${STORAGE_KEY}.endpoint`);
      set({
        endpoint: "",
        host: null,
        projects: [],
        sessions: [],
        selectedSessionId: null,
        snapshot: null,
        credential: null,
        pairing: null,
        error: null,
        streamState: "idle",
      });
    },
    connectStream: () => {
      const { endpoint, credential, selectedSessionId } = get();
      if (!credential || !endpoint || !selectedSessionId) return () => undefined;
      streamCleanup?.();
      set({ streamState: "connecting" });
      const cleanup = api.streamSession(
        endpoint,
        credential.host.id,
        selectedSessionId,
        get().lastEventId ?? undefined,
        (event) => {
          if (event.session_id && event.session_id !== get().selectedSessionId) return;
          set((state) => ({
            streamState: "live",
            lastEventId: event.event_id,
            snapshot: state.snapshot ? applyEvent(state.snapshot, event) : (event.snapshot ?? null),
            sessions: state.sessions.map((session) =>
              session.id === selectedSessionId && state.snapshot
                ? {
                    ...session,
                    state: applyEvent(state.snapshot, event).state,
                    updatedAt: applyEvent(state.snapshot, event).updatedAt,
                    preview: applyEvent(state.snapshot, event).preview,
                  }
                : session,
            ),
          }));
        },
        async (error) => {
          if (error instanceof RemoteApiError && error.status === 410) {
            try {
              await loadSnapshot(selectedSessionId, endpoint, credential);
              streamCleanup = get().connectStream();
              return;
            } catch {
              /* visible error already set */
            }
          }
          set({
            streamState:
              error instanceof RemoteApiError && error.status === 401 ? "failed" : "reconnecting",
          });
          setError(error);
        },
      );
      streamCleanup = cleanup;
      return cleanup;
    },
  };
});
