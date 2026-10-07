import { describe, expect, it } from "vitest";
import {
  applyEvent,
  contentText,
  messageText,
  nextSessionState,
  normalizePendingQuestion,
  normalizeSessionSnapshot,
  parsePairingExchange,
  parseSseFrame,
  parseTokenData,
  splitSseFrames,
} from "./protocol";
import type { SessionSnapshot, SseEnvelope } from "./types";

function baseSnapshot(overrides: Partial<SessionSnapshot> = {}): SessionSnapshot {
  return {
    id: "s1",
    title: "Session",
    kind: "primary",
    project: { id: "p1", displayName: "Project" },
    state: "idle",
    updatedAt: "2026-01-01T00:00:00.000Z",
    preview: "",
    revision: 1,
    eventSequence: 1,
    messages: [],
    pendingApproval: null,
    pendingQuestion: null,
    ...overrides,
  };
}

function envelope(sequence: number, type: string, payload: Record<string, unknown> = {}) {
  return {
    event_id: `e${sequence}`,
    sequence,
    session_revision: sequence,
    session_id: "s1",
    occurred_at: `2026-01-01T00:00:0${sequence}.000Z`,
    event_type: type,
    payload,
  } satisfies SseEnvelope;
}

describe("contentText", () => {
  it("reads strings, text fields, and nested content", () => {
    expect(contentText("hi")).toBe("hi");
    expect(contentText({ text: "a" })).toBe("a");
    expect(contentText({ content: [{ text: "a" }, "b", { content: { text: "c" } }] })).toBe("abc");
    expect(contentText({ content: { text: "x" } })).toBe("x");
  });

  it("distinguishes missing text from empty text", () => {
    expect(contentText(undefined)).toBeNull();
    expect(contentText({ other: 1 })).toBeNull();
    expect(messageText(undefined)).toBe("");
    expect(messageText({ other: 1 })).toBe("");
  });
});

describe("normalizeSessionSnapshot", () => {
  it("accepts the flat contract shape", () => {
    const snapshot = normalizeSessionSnapshot({
      sessionId: "s1",
      title: "Fix bug",
      projectId: "p1",
      revision: 4,
      eventSequence: 9,
      updatedAt: "2026-01-02T00:00:00.000Z",
      messages: [
        { messageId: "m1", role: "user", text: "hello", turnId: "t1" },
        { id: "m2", role: "other", content: [{ text: "hi " }, { text: "there" }] },
      ],
    });
    expect(snapshot).toMatchObject({
      id: "s1",
      title: "Fix bug",
      project: { id: "p1", displayName: "p1" },
      state: "idle",
      revision: 4,
      eventSequence: 9,
      preview: "hi there",
    });
    expect(snapshot.messages).toEqual([
      { id: "m1", role: "user", text: "hello", createdAt: new Date(0).toISOString(), turnId: "t1" },
      {
        id: "m2",
        role: "assistant",
        text: "hi there",
        createdAt: new Date(0).toISOString(),
        turnId: undefined,
      },
    ]);
  });

  it("accepts the legacy nested shape and an explicit project", () => {
    const project = { id: "p9", displayName: "Nine" };
    const snapshot = normalizeSessionSnapshot(
      { session: { id: "s2", revision: -3, eventSequence: 1.5 }, messages: [] },
      project,
    );
    expect(snapshot.id).toBe("s2");
    expect(snapshot.project).toBe(project);
    expect(snapshot.revision).toBe(0);
    expect(snapshot.eventSequence).toBe(0);
  });

  it("derives the waiting state from pending items", () => {
    const approval = normalizeSessionSnapshot({
      sessionId: "s1",
      pendingApproval: { approvalId: "a1", revision: 2, operation: "write file", scope: "src/" },
    });
    expect(approval.state).toBe("waiting_for_approval");
    expect(approval.pendingApproval).toEqual({
      id: "a1",
      revision: 2,
      summary: "write file",
      scope: "src/",
      detail: null,
    });
    const question = normalizeSessionSnapshot({
      sessionId: "s1",
      pendingQuestion: { questionId: "q1", questions: [{ question: "Which?", options: ["A"] }] },
    });
    expect(question.state).toBe("waiting_for_answer");
  });
});

describe("normalizePendingQuestion", () => {
  it("reads the first question and option labels", () => {
    expect(
      normalizePendingQuestion(
        {
          request_id: "q1",
          questions: [{ question: "Pick", options: [{ label: "One" }, { value: "Two" }, 3] }],
        },
        7,
      ),
    ).toEqual({ id: "q1", revision: 7, prompt: "Pick", options: ["One", "Two", "3"] });
  });

  it("returns null for non-objects", () => {
    expect(normalizePendingQuestion(null)).toBeNull();
    expect(normalizePendingQuestion("x")).toBeNull();
  });
});

describe("nextSessionState", () => {
  it("maps lifecycle events", () => {
    expect(nextSessionState("turn.queued", {}, "idle")).toBe("running");
    expect(nextSessionState("approval.requested", {}, "running")).toBe("waiting_for_approval");
    expect(nextSessionState("question.asked", {}, "running")).toBe("waiting_for_answer");
    expect(nextSessionState("turn.completed", {}, "running")).toBe("idle");
    expect(nextSessionState("turn.failed", {}, "running")).toBe("failed");
    expect(nextSessionState("assistant.delta", {}, "running")).toBe("running");
  });

  it("interprets turn.state payloads", () => {
    expect(nextSessionState("turn.state", { state: "waiting_for_approval" }, "idle")).toBe(
      "waiting_for_approval",
    );
    expect(nextSessionState("turn.state", { state: "waiting" }, "idle")).toBe("waiting_for_answer");
    expect(nextSessionState("turn.state", { state: "queued" }, "idle")).toBe("running");
    expect(nextSessionState("turn.state", { state: "completed" }, "running")).toBe("idle");
    expect(nextSessionState("turn.state", { state: "unknown" }, "running")).toBe("running");
  });
});
describe("applyEvent", () => {
  it("replaces the snapshot with an envelope snapshot", () => {
    const replacement = baseSnapshot({ id: "s1", title: "Fresh" });
    expect(
      applyEvent(baseSnapshot(), { ...envelope(2, "session.snapshot"), snapshot: replacement }),
    ).toBe(replacement);
  });

  it("appends and then upserts user messages", () => {
    let snapshot = applyEvent(
      baseSnapshot(),
      envelope(2, "message.user", { message_id: "m1", text: "hi", turn_id: "t1" }),
    );
    expect(snapshot.messages).toHaveLength(1);
    expect(snapshot.messages[0]).toMatchObject({
      id: "m1",
      role: "user",
      text: "hi",
      turnId: "t1",
    });
    snapshot = applyEvent(
      snapshot,
      envelope(3, "message.user", { message_id: "m1", text: "edit" }),
    );
    expect(snapshot.messages).toHaveLength(1);
    expect(snapshot.messages[0]?.text).toBe("edit");
  });

  it("maps tool messages to the thinking role", () => {
    const snapshot = applyEvent(baseSnapshot(), envelope(2, "message.tool", { text: "ran ls" }));
    expect(snapshot.messages[0]).toMatchObject({ id: "event-2", role: "thinking" });
  });

  it("streams deltas into one message and finalizes it", () => {
    let snapshot = applyEvent(
      baseSnapshot(),
      envelope(2, "assistant.delta", { turn_id: "t1", text: "Hel" }),
    );
    snapshot = applyEvent(snapshot, envelope(3, "assistant.delta", { turn_id: "t1", text: "lo" }));
    expect(snapshot.messages).toEqual([
      expect.objectContaining({ role: "assistant", text: "Hello", streaming: true, turnId: "t1" }),
    ]);
    expect(snapshot.preview).toBe("lo");
    snapshot = applyEvent(
      snapshot,
      envelope(4, "message.assistant", { message_id: "m9", turn_id: "t1", text: "Hello!" }),
    );
    expect(snapshot.messages).toEqual([
      expect.objectContaining({ id: "m9", text: "Hello!", turnId: "t1" }),
    ]);
    expect(snapshot.messages[0]?.streaming).toBeUndefined();
  });

  it("keeps separate turns in separate streaming messages", () => {
    let snapshot = applyEvent(
      baseSnapshot(),
      envelope(2, "assistant.delta", { turn_id: "a", text: "1" }),
    );
    snapshot = applyEvent(snapshot, envelope(3, "assistant.delta", { turn_id: "b", text: "2" }));
    expect(snapshot.messages.map((message) => message.text)).toEqual(["1", "2"]);
  });

  it("tracks approvals and questions with their revisions", () => {
    let snapshot = applyEvent(
      baseSnapshot({ state: "running" }),
      envelope(2, "approval.requested", { approval: { id: "a1", summary: "Run", scope: "npm" } }),
    );
    expect(snapshot.state).toBe("waiting_for_approval");
    expect(snapshot.pendingApproval).toEqual({
      id: "a1",
      revision: 2,
      summary: "Run",
      scope: "npm",
    });
    snapshot = applyEvent(snapshot, envelope(3, "approval.resolved"));
    expect(snapshot.pendingApproval).toBeNull();
    expect(snapshot.state).toBe("idle");
    snapshot = applyEvent(
      snapshot,
      envelope(4, "question.asked", { question: { id: "q1", prompt: "Why?" } }),
    );
    expect(snapshot.pendingQuestion).toEqual({
      id: "q1",
      revision: 4,
      prompt: "Why?",
      options: [],
    });
    snapshot = applyEvent(snapshot, envelope(5, "question.replied"));
    expect(snapshot.pendingQuestion).toBeNull();
  });

  it("never moves revision or sequence backwards", () => {
    const snapshot = applyEvent(
      baseSnapshot({ revision: 10, eventSequence: 10 }),
      envelope(3, "turn.completed"),
    );
    expect(snapshot.revision).toBe(10);
    expect(snapshot.eventSequence).toBe(10);
  });

  it("does not mutate the input snapshot", () => {
    const original = baseSnapshot();
    applyEvent(original, envelope(2, "message.user", { text: "hi" }));
    expect(original.messages).toEqual([]);
  });
});

describe("SSE frame parsing", () => {
  it("splits complete frames and keeps the remainder", () => {
    expect(splitSseFrames("id: 1\ndata: a\n\nid: 2\r\ndata: b\r\n\r\ndata: par")).toEqual({
      frames: ["id: 1\ndata: a", "id: 2\r\ndata: b"],
      rest: "data: par",
    });
  });

  it("joins multi-line data and reads the id", () => {
    expect(parseSseFrame('id: e7\nevent: x\ndata: {"a":\ndata:1}\n: comment')).toEqual({
      id: "e7",
      data: '{"a":\n1}',
    });
    expect(parseSseFrame(": keep-alive")).toEqual({ id: undefined, data: "" });
  });
});

describe("token guards", () => {
  const token = { accessToken: "a", refreshToken: "r", accessTokenExpiresAt: "2026-01-01" };

  it("keeps only the token fields", () => {
    expect(parseTokenData({ ...token, extra: true })).toEqual(token);
  });

  it("rejects malformed token responses", () => {
    expect(() => parseTokenData(undefined)).toThrow(/invalid token/);
    expect(() => parseTokenData({ ...token, refreshToken: 1 })).toThrow(/invalid token/);
  });

  it("requires a host on pairing exchange", () => {
    const host = { id: "h1", displayName: "Desk", connectionState: "connected" };
    expect(parsePairingExchange({ ...token, host })).toEqual({ ...token, host });
    expect(() => parsePairingExchange(token)).toThrow(/invalid pairing/);
  });
});
