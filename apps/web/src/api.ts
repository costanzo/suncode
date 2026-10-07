import { decryptPayload, encryptPayload } from "./crypto";
import {
  normalizeSessionSnapshot,
  parsePairingExchange,
  parseSseFrame,
  parseTokenData,
  splitSseFrames,
  type PairingExchangeData,
} from "./protocol";
import type {
  ApiError,
  CredentialState,
  Host,
  PairingPayload,
  Project,
  SessionSummary,
  SseEnvelope,
} from "./types";

export class RemoteApiError extends Error {
  constructor(
    public readonly status: number,
    public readonly details: ApiError,
  ) {
    super(details.message);
    this.name = "RemoteApiError";
  }
}

type CredentialReader = () => CredentialState | null;
type CredentialWriter = (credential: CredentialState | null) => void;

function apiError(status: number, body: unknown, fallback: string): RemoteApiError {
  const value = body as Partial<ApiError> | null;
  return new RemoteApiError(status, {
    code: value?.code ?? status,
    message: value?.message ?? fallback,
  });
}

/** Builds `/v1/mobile/hosts/{hostId}/...` with every segment URL-encoded. */
export function hostPath(hostId: string, ...segments: string[]): string {
  return ["/v1/mobile/hosts", hostId, ...segments]
    .map((part, index) => (index === 0 ? part : encodeURIComponent(part)))
    .join("/");
}

function sleep(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve) => {
    if (signal.aborted) return resolve();
    const timer = setTimeout(done, ms);
    function done() {
      clearTimeout(timer);
      signal.removeEventListener("abort", done);
      resolve();
    }
    signal.addEventListener("abort", done);
  });
}

const STREAM_RETRY_BASE_MS = 1_000;
const STREAM_RETRY_MAX_MS = 30_000;

export type StreamConnectionState = "live" | "reconnecting";

interface RequestOptions {
  method?: "GET" | "POST";
  /** JSON request body. Sent as an E2E `encPayload` when encryption is enabled. */
  json?: unknown;
  /** Whether a JSON body may be encrypted. Defaults to true. */
  encrypted?: boolean;
}

export interface StreamHandlers {
  onEvent: (event: SseEnvelope) => void;
  /** Terminal failure; the stream has stopped and the caller owns recovery. */
  onError: (error: unknown) => void;
  onStateChange?: (state: StreamConnectionState) => void;
  /** A single malformed or undecryptable frame was skipped. */
  onFrameError?: (error: unknown) => void;
}

/**
 * Client for the remote-control contract. The endpoint and host ID come from the current
 * credential, so callers only pass resource identifiers.
 */
export class RemoteApi {
  constructor(
    private readonly credentials: CredentialReader,
    private readonly updateCredentials: CredentialWriter,
  ) {}

  private async parseBody(
    response: Response,
    credential: CredentialState | null,
  ): Promise<unknown> {
    if (response.status === 204) return undefined;
    const text = await response.text();
    if (!text) return undefined;
    let value: unknown;
    try {
      value = JSON.parse(text);
    } catch {
      return text;
    }
    if (
      typeof value === "object" &&
      value !== null &&
      "encPayload" in value &&
      credential?.e2eKey
    ) {
      return decryptPayload(
        String((value as { encPayload: string }).encPayload),
        credential.e2eKey,
      );
    }
    return value;
  }

  private refreshInFlight: Promise<CredentialState> | null = null;

  /**
   * Single-flight refresh after a 401. Concurrent callers share one refresh request so the
   * rotating refresh token is only spent once. If the credential already changed since the
   * failed request was sent, the newer credential is returned without refreshing again.
   */
  private refreshAfterUnauthorized(failed: CredentialState): Promise<CredentialState> {
    if (this.refreshInFlight) return this.refreshInFlight;
    const current = this.credentials();
    if (!current) return Promise.reject(apiError(401, null, "Pairing is no longer available."));
    if (current.accessToken !== failed.accessToken) return Promise.resolve(current);
    const pending = this.refresh(current).finally(() => {
      if (this.refreshInFlight === pending) this.refreshInFlight = null;
    });
    this.refreshInFlight = pending;
    return pending;
  }

  /**
   * Refreshes after a 401 and unpairs only when the server rejected the refresh token.
   * Transport failures are rethrown unchanged so a network blip does not unpair the browser.
   */
  private async refreshOrUnpair(failed: CredentialState): Promise<CredentialState> {
    try {
      return await this.refreshAfterUnauthorized(failed);
    } catch (error) {
      if (error instanceof RemoteApiError && [400, 401, 403].includes(error.status)) {
        this.updateCredentials(null);
        throw apiError(401, null, "Your pairing has expired. Pair this browser again.");
      }
      throw error;
    }
  }

  private async refresh(credential: CredentialState): Promise<CredentialState> {
    const response = await fetch(`${credential.endpoint}/v1/mobile/auth/refresh`, {
      method: "POST",
      headers: {
        Accept: "application/json",
        "Content-Type": "application/json",
        "X-Host-Id": credential.host.id,
        "X-Request-Id": crypto.randomUUID(),
      },
      body: JSON.stringify({ refreshToken: credential.refreshToken }),
    });
    const body = await this.parseBody(response, null);
    if (!response.ok) throw apiError(response.status, body, "Refresh token rejected.");
    const next = { ...credential, ...parseTokenData(body) };
    this.updateCredentials(next);
    return next;
  }

  private async request<T>(
    path: string,
    { method = "GET", json, encrypted = true }: RequestOptions = {},
    retried = false,
  ): Promise<T> {
    const credential = this.credentials();
    if (!credential) throw apiError(401, null, "This browser is not paired with a Desktop.");
    const headers = new Headers({
      Accept: "application/json",
      "X-Request-Id": crypto.randomUUID(),
      Authorization: `Bearer ${credential.accessToken}`,
      "X-Host-Id": credential.host.id,
    });
    let body: string | undefined;
    if (json !== undefined) {
      headers.set("Content-Type", "application/json");
      const key = encrypted && credential.encryptionEnabled ? credential.e2eKey : undefined;
      body = JSON.stringify(key ? { encPayload: await encryptPayload(json, key) } : json);
    }
    const response = await fetch(`${credential.endpoint}${path}`, { method, headers, body });
    if (response.status === 401 && !retried) {
      await this.refreshOrUnpair(credential);
      return this.request<T>(path, { method, json, encrypted }, true);
    }
    const parsed = await this.parseBody(response, credential);
    if (!response.ok)
      throw apiError(response.status, parsed, response.statusText || "Remote request failed.");
    return parsed as T;
  }

  async exchangePairing(
    endpoint: string,
    payload: PairingPayload,
    deviceName: string,
  ): Promise<PairingExchangeData> {
    const response = await fetch(`${endpoint}/v1/mobile/pairings/exchange`, {
      method: "POST",
      headers: {
        Accept: "application/json",
        "Content-Type": "application/json",
        "X-Host-Id": payload.hostId,
        "X-Request-Id": crypto.randomUUID(),
      },
      body: JSON.stringify({ pairingCode: payload.code, deviceName }),
    });
    const body = await this.parseBody(response, null);
    if (!response.ok) throw apiError(response.status, body, "Pairing exchange failed.");
    return parsePairingExchange(body);
  }

  private hostId(): string {
    return this.credentials()?.host.id ?? "";
  }

  getHost() {
    return this.request<Host>(hostPath(this.hostId()));
  }
  listProjects() {
    return this.request<{ items: Project[] }>(hostPath(this.hostId(), "projects"));
  }
  listSessions(projectId?: string) {
    const query = projectId ? `?projectId=${encodeURIComponent(projectId)}` : "";
    return this.request<{ items: SessionSummary[]; nextCursor?: string | null; hasMore?: boolean }>(
      `${hostPath(this.hostId(), "sessions")}${query}`,
    );
  }
  async getSession(sessionId: string, project?: Project) {
    const value = await this.request<unknown>(hostPath(this.hostId(), "sessions", sessionId));
    return normalizeSessionSnapshot(value, project);
  }
  sendMessage(sessionId: string, text: string) {
    return this.request<void>(hostPath(this.hostId(), "sessions", sessionId, "messages"), {
      method: "POST",
      json: { text },
    });
  }
  resolveApproval(
    sessionId: string,
    approvalId: string,
    action: "allow_once" | "allow_session" | "deny",
    expectedRevision: number,
  ) {
    return this.request<void>(
      hostPath(this.hostId(), "sessions", sessionId, "approvals", approvalId),
      { method: "POST", json: { action, expectedRevision } },
    );
  }
  replyQuestion(
    sessionId: string,
    questionId: string,
    answers: string[],
    expectedRevision: number,
  ) {
    return this.request<void>(
      hostPath(this.hostId(), "sessions", sessionId, "questions", questionId, "reply"),
      { method: "POST", json: { answers, expectedRevision } },
    );
  }
  cancel(sessionId: string) {
    return this.request<void>(hostPath(this.hostId(), "sessions", sessionId, "cancel"), {
      method: "POST",
    });
  }
  retry(sessionId: string) {
    return this.request<void>(hostPath(this.hostId(), "sessions", sessionId, "retry"), {
      method: "POST",
    });
  }
  logout() {
    return this.request<void>("/v1/mobile/auth/logout", { method: "POST", encrypted: false });
  }

  /**
   * Streams one Session over SSE and reconnects with capped exponential backoff, resuming from
   * the last delivered event ID. Terminal errors (401 after a failed refresh, 404, 410, and other
   * non-retryable 4xx responses) are reported through `onError` and stop the stream; the caller
   * owns recovery for those. Individual malformed or undecryptable frames are skipped and
   * reported through `onFrameError` without closing the stream.
   */
  streamSession(
    sessionId: string,
    lastEventId: string | undefined,
    {
      onEvent,
      onError,
      onStateChange = () => undefined,
      onFrameError = (error) => console.warn("Skipped an unreadable session stream frame.", error),
    }: StreamHandlers,
  ) {
    const controller = new AbortController();
    const signal = controller.signal;
    const initial = this.credentials();
    const endpoint = initial?.endpoint ?? "";
    const hostId = initial?.host.id ?? "";
    let cursor = lastEventId;
    let wentLive = false;
    let retryAfterMs: number | undefined;

    const isTerminal = (error: unknown) =>
      error instanceof RemoteApiError &&
      error.status >= 400 &&
      error.status < 500 &&
      error.status !== 408 &&
      error.status !== 429;

    const readFrame = async (frame: string, credential: CredentialState | null) => {
      const { id: frameId, data } = parseSseFrame(frame);
      if (!data.trim()) return;
      try {
        const parsed = JSON.parse(data) as { encPayload?: string } | SseEnvelope;
        const decrypted =
          "encPayload" in parsed && parsed.encPayload && credential?.e2eKey
            ? await decryptPayload(parsed.encPayload, credential.e2eKey)
            : parsed;
        const event = decrypted as SseEnvelope;
        if (event.snapshot) {
          const snapshot = normalizeSessionSnapshot(event.snapshot, undefined);
          // The envelope's sequence/revision describe the snapshot when its body omits them.
          if (!snapshot.eventSequence) snapshot.eventSequence = Number(event.sequence) || 0;
          if (!snapshot.revision) snapshot.revision = Number(event.session_revision) || 0;
          event.snapshot = snapshot;
        }
        if (signal.aborted) return;
        onEvent(event);
      } catch (error) {
        if (!signal.aborted) onFrameError(error);
      } finally {
        // Advance the cursor even past a skipped frame so a reconnect does not replay it forever.
        if (frameId) cursor = frameId;
      }
    };

    /** Opens one connection and reads it until it ends. Throws on any failure. */
    const connectOnce = async (refreshed: boolean): Promise<void> => {
      const credential = this.credentials();
      const headers: Record<string, string> = {
        Accept: "text/event-stream",
        "X-Host-Id": hostId,
        "X-Request-Id": crypto.randomUUID(),
      };
      if (credential) headers.Authorization = `Bearer ${credential.accessToken}`;
      if (cursor) headers["Last-Event-ID"] = cursor;
      const response = await fetch(
        `${endpoint}${hostPath(hostId, "sessions", sessionId, "events")}`,
        {
          headers,
          signal,
        },
      );
      if (!response.ok || !response.body) {
        const body = await response.text();
        const error = apiError(
          response.status,
          (() => {
            try {
              return JSON.parse(body);
            } catch {
              return null;
            }
          })(),
          "Session stream failed.",
        );
        if (error.status === 401 && credential && !refreshed) {
          await this.refreshOrUnpair(credential);
          return connectOnce(true);
        }
        const retryAfter = Number(response.headers.get("Retry-After"));
        if (error.status === 429 && retryAfter > 0) retryAfterMs = retryAfter * 1_000;
        throw error;
      }
      wentLive = true;
      onStateChange("live");
      const reader = response.body.getReader();
      const decoder = new TextDecoder();
      let buffer = "";
      try {
        while (!signal.aborted) {
          const { value, done } = await reader.read();
          if (done) break;
          buffer += decoder.decode(value, { stream: true });
          const { frames, rest } = splitSseFrames(buffer);
          buffer = rest;
          for (const frame of frames) await readFrame(frame, credential);
        }
      } finally {
        reader.releaseLock();
      }
    };

    const run = async () => {
      let attempt = 0;
      while (!signal.aborted) {
        let delay: number;
        wentLive = false;
        retryAfterMs = undefined;
        try {
          await connectOnce(false);
          // Clean end of stream: reconnect promptly, resuming from the cursor.
          attempt = 0;
          delay = STREAM_RETRY_BASE_MS;
        } catch (error) {
          if (signal.aborted) return;
          if (isTerminal(error)) {
            onError(error);
            return;
          }
          // A connection that reached live delivery restarts the backoff schedule.
          if (wentLive) attempt = 0;
          delay = Math.min(STREAM_RETRY_MAX_MS, STREAM_RETRY_BASE_MS * 2 ** attempt);
          attempt += 1;
        }
        if (signal.aborted) return;
        onStateChange("reconnecting");
        // Full jitter on the upper half keeps many clients from reconnecting in lockstep.
        await sleep(retryAfterMs ?? delay / 2 + Math.random() * (delay / 2), signal);
      }
    };
    void run();
    return () => controller.abort();
  }
}
