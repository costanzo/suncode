/**
 * Pure wire normalization for the remote-control contract
 * (contracts/remote-control/mobile.openapi.yaml). Nothing here performs I/O.
 */
import type {
  Host,
  Question,
  SessionMessage,
  SessionSnapshot,
  SessionState,
  SseEnvelope,
} from "./types";

type RawSessionSnapshot = {
  session?: Record<string, unknown>;
  messages?: unknown[];
  pendingApproval?: Record<string, unknown> | null;
  pendingQuestion?: Record<string, unknown> | null;
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

/**
 * Extracts display text from a string, `{ text }`, or a (possibly nested) `content` value.
 * Returns null when the value carries no text at all.
 */
export function contentText(value: unknown): string | null {
  if (typeof value === "string") return value;
  if (!isRecord(value)) return null;
  if (typeof value.text === "string") return value.text;
  if (Array.isArray(value.content))
    return value.content.map((part) => contentText(part) ?? "").join("");
  if (value.content) return contentText(value.content);
  return null;
}

export function messageText(value: unknown): string {
  return contentText(value) ?? "";
}

export function normalizePendingQuestion(value: unknown, revision = 0): Question | null {
  if (!isRecord(value)) return null;
  const entries = Array.isArray(value.questions) ? value.questions : [value];
  const first = (entries[0] ?? {}) as Record<string, unknown>;
  const options = Array.isArray(first.options)
    ? first.options.map((option) => {
        if (isRecord(option)) {
          return String(option.label ?? option.value ?? option.description ?? "");
        }
        return String(option);
      })
    : [];
  return {
    id: String(value.request_id ?? value.questionId ?? value.id ?? ""),
    revision: Number(value.revision ?? revision),
    prompt: String(
      first.question ?? first.prompt ?? value.prompt ?? first.header ?? "SunCode needs an answer",
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
    ? raw.messages.map((item, index): SessionMessage => {
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
        };
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
export interface TokenData {
  accessToken: string;
  refreshToken: string;
  accessTokenExpiresAt: string;
}

export interface PairingExchangeData extends TokenData {
  host: Host;
}

/** Validates the contract's TokenData shape and keeps only its token fields. */
export function parseTokenData(value: unknown): TokenData {
  if (
    !isRecord(value) ||
    typeof value.accessToken !== "string" ||
    typeof value.refreshToken !== "string" ||
    typeof value.accessTokenExpiresAt !== "string"
  ) {
    throw new Error("The server returned an invalid token response.");
  }
  return {
    accessToken: value.accessToken,
    refreshToken: value.refreshToken,
    accessTokenExpiresAt: value.accessTokenExpiresAt,
  };
}

/** Validates the contract's PairingExchangeData shape. */
export function parsePairingExchange(value: unknown): PairingExchangeData {
  const token = parseTokenData(value);
  const host = (value as Record<string, unknown>).host;
  if (!isRecord(host) || typeof host.id !== "string" || typeof host.displayName !== "string") {
    throw new Error("The server returned an invalid pairing response.");
  }
  return { ...token, host: host as unknown as Host };
}

export interface SseFrame {
  id?: string;
  data: string;
}

/**
 * Splits buffered SSE text into complete frames. The trailing incomplete frame is returned as
 * `rest` so the caller can prepend it to the next chunk.
 */
export function splitSseFrames(buffer: string): { frames: string[]; rest: string } {
  const frames = buffer.split(/\r?\n\r?\n/);
  const rest = frames.pop() ?? "";
  return { frames, rest };
}

/** Parses the `id:` and `data:` fields of one SSE frame. Other fields are ignored. */
export function parseSseFrame(frame: string): SseFrame {
  let id: string | undefined;
  const dataLines: string[] = [];
  for (const line of frame.split(/\r?\n/)) {
    if (line.startsWith("data:")) dataLines.push(line.slice(5).replace(/^ /, ""));
    else if (line.startsWith("id:")) id = line.slice(3).trim();
  }
  return { id, data: dataLines.join("\n") };
}

function findStreamingMessageIndex(messages: SessionMessage[], turnId: string): number {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const message = messages[index];
    if (message?.role === "assistant" && message.streaming && message.turnId === turnId) {
      return index;
    }
  }
  return -1;
}

export function nextSessionState(
  type: string,
  payload: Record<string, unknown>,
  current: SessionState,
): SessionState {
  if (type === "turn.queued" || type === "provider.exchange.started") return "running";
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

function upsertMessage(messages: SessionMessage[], next: SessionMessage): SessionMessage[] {
  return messages.some((item) => item.id === next.id)
    ? messages.map((item) => (item.id === next.id ? next : item))
    : [...messages, next];
}

/** Applies one SSE envelope to a snapshot. A `snapshot` envelope replaces it entirely. */
export function applyEvent(snapshot: SessionSnapshot, event: SseEnvelope): SessionSnapshot {
  if (event.snapshot) return event.snapshot;
  const type = event.event_type ?? "";
  const payload = event.payload ?? {};
  let messages = snapshot.messages;
  const text = contentText(payload.message ?? payload.text);
  const messageId = String(payload.message_id ?? payload.id ?? `event-${event.sequence}`);
  const turnIdValue = payload.turn_id ?? payload.turnId;
  const turnId = turnIdValue == null ? undefined : String(turnIdValue);
  const createdAt = () =>
    String(payload.occurred_at ?? event.occurred_at ?? new Date().toISOString());
  if ((type === "message.user" || type === "message.tool") && text) {
    messages = upsertMessage(messages, {
      id: messageId,
      role: type === "message.user" ? "user" : "thinking",
      text,
      createdAt: createdAt(),
      turnId,
    });
  } else if (type === "assistant.delta" && text) {
    const streamTurnId = turnId ?? `event-${event.sequence}`;
    const streamingIndex = findStreamingMessageIndex(messages, streamTurnId);
    if (streamingIndex >= 0) {
      messages = messages.map((item, index) =>
        index === streamingIndex ? { ...item, text: `${item.text}${text}` } : item,
      );
    } else {
      messages = [
        ...messages,
        {
          id: `assistant-stream-${streamTurnId}-${event.sequence}`,
          role: "assistant",
          text,
          createdAt: event.occurred_at ?? new Date().toISOString(),
          turnId: streamTurnId,
          streaming: true,
        },
      ];
    }
  } else if (type === "message.assistant" && text) {
    const streamingIndex = turnId ? findStreamingMessageIndex(messages, turnId) : -1;
    const next: SessionMessage = {
      id: messageId,
      role: "assistant",
      text,
      createdAt: createdAt(),
      turnId,
    };
    messages =
      streamingIndex >= 0
        ? messages.map((item, index) => (index === streamingIndex ? next : item))
        : upsertMessage(messages, next);
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
    pendingQuestion = normalizePendingQuestion(payload.question ?? payload, event.session_revision);
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
