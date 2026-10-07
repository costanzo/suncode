import { decryptPayload, encryptPayload } from "./crypto";
import type {
  ApiError,
  CredentialState,
  Host,
  PairingPayload,
  Project,
  Question,
  SessionSnapshot,
  SessionSummary,
  SseEnvelope,
} from "./types";

type RawSessionSnapshot = {
  session?: Record<string, unknown>;
  messages?: unknown[];
  pendingApproval?: Record<string, unknown> | null;
  pendingQuestion?: Record<string, unknown> | null;
};

function messageText(value: unknown): string {
  if (typeof value === "string") return value;
  if (!value || typeof value !== "object") return "";
  const object = value as Record<string, unknown>;
  if (typeof object.text === "string") return object.text;
  if (Array.isArray(object.content)) return object.content.map(messageText).join("");
  if (object.content) return messageText(object.content);
  return "";
}

export function normalizePendingQuestion(value: unknown, revision = 0): Question | null {
  if (!value || typeof value !== "object") return null;
  const source = value as Record<string, unknown>;
  const entries = Array.isArray(source.questions) ? source.questions : [source];
  const first = (entries[0] ?? {}) as Record<string, unknown>;
  const options = Array.isArray(first.options)
    ? first.options.map((option) => {
        if (option && typeof option === "object") {
          const item = option as Record<string, unknown>;
          return String(item.label ?? item.value ?? item.description ?? "");
        }
        return String(option);
      })
    : [];
  return {
    id: String(source.request_id ?? source.questionId ?? source.id ?? ""),
    revision: Number(source.revision ?? revision),
    prompt: String(
      first.question ?? first.prompt ?? source.prompt ?? first.header ?? "SunCode needs an answer",
    ),
    options,
  };
}

function nonNegativeInteger(value: unknown): number {
  const number = Number(value);
  return Number.isInteger(number) && number >= 0 ? number : 0;
}

export function normalizeSessionSnapshot(
  value: unknown,
  project?: { id: string; displayName: string },
): SessionSnapshot {
  const raw = (value ?? {}) as RawSessionSnapshot & Record<string, unknown>;
  // The contract's SessionDetailData is flat; older payloads nest fields under `session`.
  const session = raw.session ?? raw;
  const rawProject = (session.project ?? null) as { id?: unknown; displayName?: unknown } | null;
  const projectId = String(session.projectId ?? rawProject?.id ?? project?.id ?? "");
  const sessionProject = project ?? {
    id: projectId,
    displayName: String(rawProject?.displayName ?? (projectId || "Project")),
  };
  const messages = Array.isArray(raw.messages)
    ? raw.messages.map((item, index) => {
        const message = (item ?? {}) as Record<string, unknown>;
        const role =
          message.role === "user" || message.role === "thinking" ? message.role : "assistant";
        return {
          id: String(message.messageId ?? message.id ?? `message-${index}`),
          role,
          text: messageText(message),
          createdAt: String(message.createdAt ?? new Date(0).toISOString()),
          turnId:
            message.turnId == null && message.turn_id == null
              ? undefined
              : String(message.turnId ?? message.turn_id),
        } as SessionSnapshot["messages"][number];
      })
    : [];
  const approval = raw.pendingApproval;
  const question = normalizePendingQuestion(raw.pendingQuestion);
  return {
    id: String(session.sessionId ?? session.id ?? ""),
    title: String(session.title ?? "New session"),
    kind: "primary",
    project: sessionProject,
    state: approval ? "waiting_for_approval" : question ? "waiting_for_answer" : "idle",
    updatedAt: String(session.updatedAt ?? session.lastActivityAt ?? new Date(0).toISOString()),
    preview: messages.at(-1)?.text ?? "",
    revision: nonNegativeInteger(session.revision ?? raw.revision),
    eventSequence: nonNegativeInteger(session.eventSequence ?? raw.eventSequence),
    messages,
    pendingApproval: approval
      ? {
          id: String(approval.approvalId ?? approval.id ?? "approval"),
          revision: Number(approval.revision ?? 0),
          summary: String(approval.summary ?? approval.operation ?? "Approval required"),
          scope: String(approval.scope ?? ""),
          detail: approval.detail == null ? null : String(approval.detail),
        }
      : null,
    pendingQuestion: question,
  };
}

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

/** Recovers the pairing endpoint (which may include a path prefix) from an API URL. */
function endpointFromUrl(url: string): string {
  const index = url.indexOf("/v1/mobile/");
  return index >= 0 ? url.slice(0, index) : new URL(url).origin;
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
  private refreshAfterUnauthorized(
    endpoint: string,
    failed: CredentialState,
  ): Promise<CredentialState> {
    if (this.refreshInFlight) return this.refreshInFlight;
    const current = this.credentials();
    if (!current) return Promise.reject(apiError(401, null, "Pairing is no longer available."));
    if (current.accessToken !== failed.accessToken) return Promise.resolve(current);
    const pending = this.refresh(endpoint, current).finally(() => {
      if (this.refreshInFlight === pending) this.refreshInFlight = null;
    });
    this.refreshInFlight = pending;
    return pending;
  }

  /**
   * Refreshes after a 401 and unpairs only when the server rejected the refresh token.
   * Transport failures are rethrown unchanged so a network blip does not unpair the browser.
   */
  private async refreshOrUnpair(
    endpoint: string,
    failed: CredentialState,
  ): Promise<CredentialState> {
    try {
      return await this.refreshAfterUnauthorized(endpoint, failed);
    } catch (error) {
      if (error instanceof RemoteApiError && [400, 401, 403].includes(error.status)) {
        this.updateCredentials(null);
        throw apiError(401, null, "Your pairing has expired. Pair this browser again.");
      }
      throw error;
    }
  }

  private async refresh(endpoint: string, credential: CredentialState): Promise<CredentialState> {
    const response = await fetch(`${endpoint}/v1/mobile/auth/refresh`, {
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
    const token = body as {
      accessToken: string;
      refreshToken: string;
      accessTokenExpiresAt: string;
    };
    const next = { ...credential, ...token };
    this.updateCredentials(next);
    return next;
  }

  private async request<T>(
    url: string,
    init: RequestInit = {},
    encrypted = true,
    retried = false,
  ): Promise<T> {
    const credential = this.credentials();
    const headers = new Headers(init.headers);
    headers.set("Accept", "application/json");
    headers.set("X-Request-Id", crypto.randomUUID());
    if (credential) {
      headers.set("Authorization", `Bearer ${credential.accessToken}`);
      headers.set("X-Host-Id", credential.host.id);
    }
    let body = init.body;
    if (body && encrypted && credential?.e2eKey && credential.encryptionEnabled) {
      const plain = typeof body === "string" ? JSON.parse(body) : body;
      body = JSON.stringify({ encPayload: await encryptPayload(plain, credential.e2eKey) });
      headers.set("Content-Type", "application/json");
    }
    const response = await fetch(url, { ...init, headers, body });
    if (response.status === 401 && credential && !retried && !url.endsWith("/auth/refresh")) {
      await this.refreshOrUnpair(endpointFromUrl(url), credential);
      return this.request<T>(url, init, encrypted, true);
    }
    const parsed = await this.parseBody(response, credential);
    if (!response.ok)
      throw apiError(response.status, parsed, response.statusText || "Remote request failed.");
    return parsed as T;
  }

  async exchangePairing(endpoint: string, payload: PairingPayload, deviceName: string) {
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
    return body as {
      accessToken: string;
      refreshToken: string;
      accessTokenExpiresAt: string;
      host: Host;
    };
  }

  getHost(endpoint: string, hostId: string) {
    return this.request<Host>(`${endpoint}/v1/mobile/hosts/${encodeURIComponent(hostId)}`);
  }
  listProjects(endpoint: string, hostId: string) {
    return this.request<{ items: Project[] }>(
      `${endpoint}/v1/mobile/hosts/${encodeURIComponent(hostId)}/projects`,
    );
  }
  listSessions(endpoint: string, hostId: string, projectId?: string) {
    return this.request<{ items: SessionSummary[]; nextCursor?: string | null; hasMore?: boolean }>(
      `${endpoint}/v1/mobile/hosts/${encodeURIComponent(hostId)}/sessions${projectId ? `?projectId=${encodeURIComponent(projectId)}` : ""}`,
    );
  }
  async getSession(endpoint: string, hostId: string, sessionId: string, project?: Project) {
    const value = await this.request<unknown>(
      `${endpoint}/v1/mobile/hosts/${encodeURIComponent(hostId)}/sessions/${encodeURIComponent(sessionId)}`,
    );
    return normalizeSessionSnapshot(value, project);
  }
  sendMessage(endpoint: string, hostId: string, sessionId: string, text: string) {
    return this.request<void>(
      `${endpoint}/v1/mobile/hosts/${encodeURIComponent(hostId)}/sessions/${encodeURIComponent(sessionId)}/messages`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ text }),
      },
    );
  }
  resolveApproval(
    endpoint: string,
    hostId: string,
    sessionId: string,
    approvalId: string,
    action: "allow_once" | "allow_session" | "deny",
    expectedRevision: number,
  ) {
    return this.request<void>(
      `${endpoint}/v1/mobile/hosts/${encodeURIComponent(hostId)}/sessions/${encodeURIComponent(sessionId)}/approvals/${encodeURIComponent(approvalId)}`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ action, expectedRevision }),
      },
    );
  }
  replyQuestion(
    endpoint: string,
    hostId: string,
    sessionId: string,
    questionId: string,
    answers: string[],
    expectedRevision: number,
  ) {
    return this.request<void>(
      `${endpoint}/v1/mobile/hosts/${encodeURIComponent(hostId)}/sessions/${encodeURIComponent(sessionId)}/questions/${encodeURIComponent(questionId)}/reply`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ answers, expectedRevision }),
      },
    );
  }
  cancel(endpoint: string, hostId: string, sessionId: string) {
    return this.request<void>(
      `${endpoint}/v1/mobile/hosts/${encodeURIComponent(hostId)}/sessions/${encodeURIComponent(sessionId)}/cancel`,
      { method: "POST" },
    );
  }
  retry(endpoint: string, hostId: string, sessionId: string) {
    return this.request<void>(
      `${endpoint}/v1/mobile/hosts/${encodeURIComponent(hostId)}/sessions/${encodeURIComponent(sessionId)}/retry`,
      { method: "POST" },
    );
  }
  logout(endpoint: string, hostId: string) {
    return this.request<void>(`${endpoint}/v1/mobile/auth/logout`, { method: "POST" }, false);
  }

  /**
   * Streams one Session over SSE and reconnects with capped exponential backoff, resuming from
   * the last delivered event ID. Terminal errors (401 after a failed refresh, 404, 410, and other
   * non-retryable 4xx responses) are reported through `onError` and stop the stream; the caller
   * owns recovery for those. Individual malformed or undecryptable frames are skipped and
   * reported through `onFrameError` without closing the stream.
   */
  streamSession(
    endpoint: string,
    hostId: string,
    sessionId: string,
    lastEventId: string | undefined,
    onEvent: (event: SseEnvelope) => void,
    onError: (error: unknown) => void,
    onStateChange: (state: StreamConnectionState) => void = () => undefined,
    onFrameError: (error: unknown) => void = (error) =>
      console.warn("Skipped an unreadable session stream frame.", error),
  ) {
    const controller = new AbortController();
    const signal = controller.signal;
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
      let frameId: string | undefined;
      const dataLines: string[] = [];
      for (const line of frame.split(/\r?\n/)) {
        if (line.startsWith("data:")) dataLines.push(line.slice(5).replace(/^ /, ""));
        else if (line.startsWith("id:")) frameId = line.slice(3).trim();
      }
      const data = dataLines.join("\n");
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
        `${endpoint}/v1/mobile/hosts/${encodeURIComponent(hostId)}/sessions/${encodeURIComponent(sessionId)}/events`,
        { headers, signal },
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
          await this.refreshOrUnpair(endpoint, credential);
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
          const frames = buffer.split(/\r?\n\r?\n/);
          buffer = frames.pop() ?? "";
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
