import { afterEach, describe, expect, it, vi } from "vitest";
import { hostPath, RemoteApi, RemoteApiError } from "./api";
import { decryptPayload, importPairingKey } from "./crypto";
import type { CredentialState } from "./types";

const ENDPOINT = "https://relay.example/prefix";

function credential(overrides: Partial<CredentialState> = {}): CredentialState {
  return {
    endpoint: ENDPOINT,
    host: { id: "host/1", displayName: "Desk", connectionState: "connected" },
    accessToken: "old-access",
    refreshToken: "old-refresh",
    accessTokenExpiresAt: "2026-01-01T00:00:00.000Z",
    encryptionEnabled: false,
    ...overrides,
  };
}

/** A RemoteApi backed by an in-memory credential, as the store wires it. */
function harness(initial: CredentialState) {
  let current: CredentialState | null = initial;
  const writes: (CredentialState | null)[] = [];
  const api = new RemoteApi(
    () => current,
    (next) => {
      current = next;
      writes.push(next);
    },
  );
  /** Replaces the credential as another caller's completed refresh would. */
  const replace = (next: CredentialState) => (current = next);
  return { api, writes, replace, current: () => current };
}

const json = (status: number, body: unknown) =>
  new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });

type Call = { url: string; init: RequestInit };

function stubFetch(handler: (call: Call) => Response | Promise<Response>) {
  const calls: Call[] = [];
  vi.stubGlobal(
    "fetch",
    vi.fn(async (url: string, init: RequestInit = {}) => {
      const call = { url, init };
      calls.push(call);
      return handler(call);
    }),
  );
  return calls;
}

const authorization = (call: Call) => new Headers(call.init.headers).get("Authorization");
const isRefresh = (call: Call) => call.url.endsWith("/v1/mobile/auth/refresh");

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("hostPath", () => {
  it("encodes every segment", () => {
    expect(hostPath("host/1", "sessions", "a b", "events")).toBe(
      "/v1/mobile/hosts/host%2F1/sessions/a%20b/events",
    );
  });
});

describe("refresh single-flight", () => {
  it("shares one refresh between concurrent 401s and retries both requests", async () => {
    const { api, current } = harness(credential());
    let releaseRefresh!: () => void;
    const refreshGate = new Promise<void>((resolve) => (releaseRefresh = resolve));
    const calls = stubFetch(async (call) => {
      if (isRefresh(call)) {
        await refreshGate;
        return json(200, {
          accessToken: "new-access",
          refreshToken: "new-refresh",
          accessTokenExpiresAt: "2026-01-02T00:00:00.000Z",
        });
      }
      if (authorization(call) === "Bearer old-access")
        return json(401, { code: 401, message: "expired" });
      return json(200, { items: [{ id: "p1", displayName: "P" }] });
    });

    const first = api.listProjects();
    const second = api.listSessions();
    await vi.waitFor(() => expect(calls.filter(isRefresh)).toHaveLength(1));
    releaseRefresh();
    await expect(Promise.all([first, second])).resolves.toHaveLength(2);

    expect(calls.filter(isRefresh)).toHaveLength(1);
    expect(JSON.parse(String(calls.find(isRefresh)?.init.body))).toEqual({
      refreshToken: "old-refresh",
    });
    const retried = calls.filter(
      (call) => !isRefresh(call) && authorization(call) === "Bearer new-access",
    );
    expect(retried).toHaveLength(2);
    expect(current()).toMatchObject({ accessToken: "new-access", refreshToken: "new-refresh" });
    expect(current()?.endpoint).toBe(ENDPOINT);
  });

  it("reuses a credential refreshed after the failing request was sent", async () => {
    const { api, replace } = harness(credential());
    const calls = stubFetch((call) => {
      if (isRefresh(call)) throw new Error("must not refresh");
      if (authorization(call) === "Bearer old-access") {
        // Simulate another caller finishing a refresh while this request was in flight.
        replace(credential({ accessToken: "newer-access" }));
        return json(401, {});
      }
      return json(200, { id: "host/1" });
    });
    await api.getHost();
    expect(calls.map(authorization)).toEqual(["Bearer old-access", "Bearer newer-access"]);
  });

  it("unpairs when the server rejects the refresh token", async () => {
    const { api, current } = harness(credential());
    stubFetch((call) => (isRefresh(call) ? json(401, { message: "revoked" }) : json(401, {})));
    const error = await api.getHost().catch((reason: unknown) => reason);
    expect(error).toBeInstanceOf(RemoteApiError);
    expect((error as RemoteApiError).message).toMatch(/Pair this browser again/);
    expect(current()).toBeNull();
  });

  it("keeps the pairing when the refresh fails in transport", async () => {
    const { api, current } = harness(credential());
    stubFetch((call) => {
      if (isRefresh(call)) throw new TypeError("network down");
      return json(401, {});
    });
    await expect(api.getHost()).rejects.toThrow("network down");
    expect(current()?.accessToken).toBe("old-access");
  });

  it("rejects a malformed refresh response without storing it", async () => {
    const { api, writes } = harness(credential());
    stubFetch((call) => (isRefresh(call) ? json(200, { accessToken: 1 }) : json(401, {})));
    await expect(api.getHost()).rejects.toThrow(/invalid token/);
    expect(writes).toEqual([]);
  });
});

describe("request bodies", () => {
  it("sends plain JSON when encryption is off", async () => {
    const { api } = harness(credential());
    const calls = stubFetch(() => new Response(null, { status: 204 }));
    await api.sendMessage("s1", "hello");
    expect(calls[0]?.url).toBe(`${ENDPOINT}/v1/mobile/hosts/host%2F1/sessions/s1/messages`);
    expect(calls[0]?.init.method).toBe("POST");
    expect(JSON.parse(String(calls[0]?.init.body))).toEqual({ text: "hello" });
  });

  it("encrypts the JSON body when encryption is on", async () => {
    const e2eKey = await importPairingKey("AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE");
    const { api } = harness(credential({ e2eKey, encryptionEnabled: true }));
    const calls = stubFetch(() => new Response(null, { status: 204 }));
    await api.resolveApproval("s1", "a1", "deny", 3);
    const sent = JSON.parse(String(calls[0]?.init.body)) as { encPayload: string };
    expect(await decryptPayload(sent.encPayload, e2eKey)).toEqual({
      action: "deny",
      expectedRevision: 3,
    });
  });
});

describe("streamSession", () => {
  /** An SSE body delivered in two chunks split mid-frame, to exercise buffering. */
  const sse = (text: string) =>
    new Response(
      new ReadableStream<Uint8Array>({
        start(controller) {
          const encoder = new TextEncoder();
          controller.enqueue(encoder.encode(text.slice(0, 7)));
          controller.enqueue(encoder.encode(text.slice(7)));
          controller.close();
        },
      }),
      { status: 200, headers: { "Content-Type": "text/event-stream" } },
    );

  it("delivers events, skips bad frames, and resumes from the last frame ID", async () => {
    // Zero jitter keeps the reconnect delay at half the 1s base.
    vi.spyOn(Math, "random").mockReturnValue(0);
    const { api } = harness(credential());
    let connection = 0;
    const calls = stubFetch(() => {
      connection += 1;
      if (connection > 1) return json(404, { message: "gone" });
      return sse(
        'id: e1\ndata: {"event_id":"e1","sequence":1,"session_revision":1,"session_id":"s1"}\n\n' +
          "id: e2\ndata: not-json\n\n",
      );
    });
    const events: string[] = [];
    const frameErrors: unknown[] = [];
    const states: string[] = [];
    const error = await new Promise<unknown>((resolve) => {
      api.streamSession("s1", undefined, {
        onEvent: (event) => events.push(event.event_id),
        onError: resolve,
        onStateChange: (state) => states.push(state),
        onFrameError: (frameError) => frameErrors.push(frameError),
      });
    });
    vi.restoreAllMocks();
    expect(events).toEqual(["e1"]);
    expect(frameErrors).toHaveLength(1);
    expect(states).toEqual(["live", "reconnecting"]);
    expect((error as RemoteApiError).status).toBe(404);
    expect(calls[0]?.url).toBe(`${ENDPOINT}/v1/mobile/hosts/host%2F1/sessions/s1/events`);
    expect(new Headers(calls[0]?.init.headers).get("Last-Event-ID")).toBeNull();
    // The cursor advanced past the skipped frame.
    expect(new Headers(calls[1]?.init.headers).get("Last-Event-ID")).toBe("e2");
  });

  it("stops without reconnecting after cleanup", async () => {
    const { api } = harness(credential());
    const calls = stubFetch(() => sse("data: {}\n\n"));
    const stop = api.streamSession("s1", "cursor-1", {
      onEvent: () => undefined,
      onError: () => undefined,
    });
    stop();
    await new Promise((resolve) => setTimeout(resolve, 20));
    expect(calls.length).toBeLessThanOrEqual(1);
    expect(new Headers(calls[0]?.init.headers).get("Last-Event-ID")).toBe("cursor-1");
  });
});
