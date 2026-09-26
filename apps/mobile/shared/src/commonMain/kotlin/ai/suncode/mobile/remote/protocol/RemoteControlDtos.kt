package ai.suncode.mobile.remote.protocol

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject

/** The common response envelope used by the Remote Server HTTP API. */
@Serializable
data class ApiBaseRet<T>(
    val code: Int = 0,
    val message: String? = null,
    val data: T? = null,
)

@Serializable
data class HealthData(
    val status: String,
    val serverTime: String,
)

@Serializable
data class PairingExchangeRequest(
    val pairingPayload: String,
    val deviceName: String,
    val devicePublicKey: String? = null,
    val clientNonce: String? = null,
)

@Serializable
data class PairingExchangeData(
    val accessToken: String,
    val refreshToken: String,
    val accessTokenExpiresAt: String,
    val host: HostDto,
)

@Serializable
data class RefreshTokenRequest(
    val refreshToken: String,
)

@Serializable
data class TokenData(
    val accessToken: String,
    val refreshToken: String,
    val accessTokenExpiresAt: String,
)

@Serializable
data class HostDto(
    val id: String,
    val displayName: String,
    val endpoint: String,
    val connectionState: String,
    val projectCount: Int,
    val activeSessionCount: Int,
    val lastSeenAt: String? = null,
    val desktopVersion: String? = null,
    val protocolVersion: String? = null,
)

@Serializable
data class ProjectDto(
    val id: String,
    val displayName: String,
    val relativeRoot: String? = null,
    val activeSessionCount: Int,
)

@Serializable
data class ProjectsData(
    val items: List<ProjectDto>,
)

@Serializable
data class HostRefDto(
    val id: String,
    val displayName: String,
)

@Serializable
data class ProjectRefDto(
    val id: String,
    val displayName: String,
)

@Serializable
data class SessionSummaryDto(
    val id: String,
    val title: String,
    val kind: String,
    val host: HostRefDto,
    val project: ProjectRefDto,
    val state: String,
    val updatedAt: String,
    val preview: String,
    val pendingApproval: Boolean = false,
    val pendingQuestion: Boolean = false,
)

@Serializable
data class SessionPageData(
    val items: List<SessionSummaryDto>,
    val nextCursor: String? = null,
    val hasMore: Boolean,
)

@Serializable
data class MessageDto(
    val id: String,
    val role: String,
    val text: String,
    val createdAt: String,
)

@Serializable
data class ApprovalRequestDto(
    val id: String,
    val revision: Int,
    val risk: String,
    val summary: String,
    val detail: String? = null,
    val createdAt: String,
)

@Serializable
data class QuestionRequestDto(
    val id: String,
    val revision: Int,
    val prompt: String,
    val options: List<String>,
    val allowsFreeText: Boolean = false,
    val createdAt: String,
)

@Serializable
data class SessionDetailDto(
    val id: String,
    val title: String,
    val kind: String,
    val host: HostRefDto,
    val project: ProjectRefDto,
    val state: String,
    val updatedAt: String,
    val preview: String,
    val pendingApproval: ApprovalRequestDto? = null,
    val pendingQuestion: QuestionRequestDto? = null,
    val revision: Int,
    val archived: Boolean,
    val messages: List<MessageDto>,
)

@Serializable
data class CreateSessionRequest(
    val hostId: String,
    val projectId: String,
    val title: String? = null,
    val firstMessage: String? = null,
)

@Serializable
data class SendMessageRequest(
    val text: String,
    val clientMessageId: String? = null,
)

@Serializable
data class ApprovalResolutionRequest(
    val action: String,
    val expectedRevision: Int,
)

@Serializable
data class QuestionReplyRequest(
    val answers: List<String>,
    val expectedRevision: Int,
)

@Serializable
data class CommandAcceptedData(
    val requestId: String,
    val acceptedAt: String,
)

@Serializable
data class SyncData(
    val cursor: String,
    val resetRequired: Boolean,
    val hosts: List<HostDto>,
    val sessions: List<SessionSummaryDto>,
    val removedSessionIds: List<String>,
    val hasMore: Boolean = false,
)

/**
 * Transport envelope for the server-to-Mobile stream. Payload stays JSON here so new
 * event types can be received before the reducer learns how to project them.
 */
@Serializable
data class AgentEventEnvelope(
    @SerialName("session_id") val sessionId: String,
    @SerialName("occurred_at") val occurredAt: String,
    @SerialName("event_type") val eventType: String,
    val payload: JsonObject,
)

object RemoteEventTypes {
    const val TURN_STATE = "turn.state"
    const val MESSAGE_USER = "message.user"
    const val MESSAGE_ASSISTANT = "message.assistant"
    const val ASSISTANT_DELTA = "assistant.delta"
    const val APPROVAL_REQUESTED = "approval.requested"
    const val APPROVAL_RESOLVED = "approval.resolved"
    const val QUESTION_ASKED = "question.asked"
    const val QUESTION_REPLIED = "question.replied"
    const val QUESTION_REJECTED = "question.rejected"
    const val TURN_COMPLETED = "turn.completed"
}
