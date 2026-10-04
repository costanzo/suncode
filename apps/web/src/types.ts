export type ConnectionState = "connected" | "degraded" | "offline" | "unauthorized";
export type SessionState =
  "idle" | "running" | "waiting_for_approval" | "waiting_for_answer" | "failed";

export interface Host {
  id: string;
  displayName: string;
  connectionState: ConnectionState;
  agentVersion?: string;
  lastSeenAt?: string;
}

export interface Project {
  id: string;
  displayName: string;
  relativeRoot?: string;
  activeSessionCount?: number;
}

export interface SessionSummary {
  id: string;
  title: string;
  kind: "primary";
  project: { id: string; displayName: string };
  state: SessionState;
  updatedAt: string;
  preview?: string;
  pendingApproval?: boolean;
  pendingQuestion?: boolean;
}

export interface SessionMessage {
  id: string;
  role: "user" | "assistant" | "thinking";
  text: string;
  createdAt: string;
}

export interface Approval {
  id: string;
  revision: number;
  summary: string;
  scope: string;
  detail?: string | null;
}

export interface Question {
  id: string;
  revision: number;
  prompt: string;
  options: string[];
}

export interface SessionSnapshot extends Omit<
  SessionSummary,
  "pendingApproval" | "pendingQuestion"
> {
  revision: number;
  eventSequence: number;
  messages: SessionMessage[];
  pendingApproval: Approval | null;
  pendingQuestion: Question | null;
}

export interface PairingPayload {
  endpoint: string;
  hostId: string;
  code: string;
  key?: string;
  e2e: boolean;
}

export interface CredentialState {
  host: Host;
  accessToken: string;
  refreshToken: string;
  accessTokenExpiresAt: string;
  e2eKey?: CryptoKey;
  e2eKeyRaw?: string;
}

export interface ApiError {
  code: number;
  message: string;
}

export interface SseEnvelope {
  event_id: string;
  sequence: number;
  session_revision: number;
  session_id: string;
  occurred_at?: string;
  event_type?: string;
  payload?: Record<string, unknown>;
  snapshot?: SessionSnapshot;
}
