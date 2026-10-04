import { decryptPayload, encryptPayload } from "./crypto";
import type {
  ApiError,
  CredentialState,
  Host,
  PairingPayload,
  Project,
  SessionSnapshot,
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
    if (body && encrypted && credential?.e2eKey) {
      const plain = typeof body === "string" ? JSON.parse(body) : body;
      body = JSON.stringify({ encPayload: await encryptPayload(plain, credential.e2eKey) });
      headers.set("Content-Type", "application/json");
    }
    const response = await fetch(url, { ...init, headers, body });
    if (response.status === 401 && credential && !retried && !url.endsWith("/auth/refresh")) {
      try {
        await this.refresh(new URL(url).origin, credential);
      } catch {
        this.updateCredentials(null);
        throw apiError(401, null, "Your pairing has expired. Pair this browser again.");
      }
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
  getSession(endpoint: string, hostId: string, sessionId: string) {
    return this.request<SessionSnapshot>(
      `${endpoint}/v1/mobile/hosts/${encodeURIComponent(hostId)}/sessions/${encodeURIComponent(sessionId)}`,
    );
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

  streamSession(
    endpoint: string,
    hostId: string,
    sessionId: string,
    lastEventId: string | undefined,
    onEvent: (event: SseEnvelope) => void,
    onError: (error: unknown) => void,
  ) {
    const controller = new AbortController();
    const connect = async (cursor: string | undefined, refreshed: boolean): Promise<void> => {
      try {
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
          { headers, signal: controller.signal },
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
            await this.refresh(endpoint, credential);
            return connect(cursor, true);
          }
          throw error;
        }
        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = "";
        while (!controller.signal.aborted) {
          const { value, done } = await reader.read();
          if (done) break;
          buffer += decoder.decode(value, { stream: true });
          const frames = buffer.split(/\r?\n\r?\n/);
          buffer = frames.pop() ?? "";
          for (const frame of frames) {
            const data = frame
              .split(/\r?\n/)
              .filter((line) => line.startsWith("data:"))
              .map((line) => line.slice(5).trim())
              .join("\n");
            if (!data) continue;
            const parsed = JSON.parse(data) as { encPayload?: string } | SseEnvelope;
            const decrypted =
              "encPayload" in parsed && parsed.encPayload && credential?.e2eKey
                ? await decryptPayload(parsed.encPayload, credential.e2eKey)
                : parsed;
            onEvent(decrypted as SseEnvelope);
          }
        }
      } catch (error) {
        if (!controller.signal.aborted) onError(error);
      }
    };
    void connect(lastEventId, false);
    return () => controller.abort();
  }
}
