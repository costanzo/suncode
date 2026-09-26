package ai.suncode.mobile.remote

import ai.suncode.mobile.remote.protocol.AgentEventEnvelope
import ai.suncode.mobile.remote.protocol.ApiBaseRet
import ai.suncode.mobile.remote.protocol.ApprovalResolutionRequest
import ai.suncode.mobile.remote.protocol.CommandAcceptedData
import ai.suncode.mobile.remote.protocol.CreateSessionRequest
import ai.suncode.mobile.remote.protocol.HealthData
import ai.suncode.mobile.remote.protocol.HostDto
import ai.suncode.mobile.remote.protocol.PairingExchangeData
import ai.suncode.mobile.remote.protocol.PairingExchangeRequest
import ai.suncode.mobile.remote.protocol.ProjectsData
import ai.suncode.mobile.remote.protocol.QuestionReplyRequest
import ai.suncode.mobile.remote.protocol.RefreshTokenRequest
import ai.suncode.mobile.remote.protocol.SendMessageRequest
import ai.suncode.mobile.remote.protocol.SessionDetailDto
import ai.suncode.mobile.remote.protocol.SessionPageData
import ai.suncode.mobile.remote.protocol.SyncData
import ai.suncode.mobile.remote.protocol.TokenData
import kotlinx.coroutines.flow.Flow

/**
 * Typed boundary for the Mobile-facing Remote Control API.
 *
 * Implementations own HTTP authentication, request headers, response status handling,
 * and WebSocket lifecycle. The UI and domain layer never construct protocol URLs or
 * envelopes directly.
 */
interface RemoteControlClient {
    suspend fun health(): ApiBaseRet<HealthData>

    suspend fun exchangePairing(request: PairingExchangeRequest): ApiBaseRet<PairingExchangeData>

    suspend fun refreshToken(request: RefreshTokenRequest): ApiBaseRet<TokenData>

    suspend fun logout(idempotencyKey: String)

    suspend fun getHost(hostId: String): ApiBaseRet<HostDto>

    suspend fun listProjects(hostId: String): ApiBaseRet<ProjectsData>

    suspend fun listSessions(
        hostId: String? = null,
        projectId: String? = null,
        cursor: String? = null,
        limit: Int? = null,
    ): ApiBaseRet<SessionPageData>

    /** The v1 endpoint returns HTTP 201 with no response body. */
    suspend fun createSession(request: CreateSessionRequest, idempotencyKey: String)

    suspend fun getSession(sessionId: String): ApiBaseRet<SessionDetailDto>

    suspend fun sendMessage(
        sessionId: String,
        request: SendMessageRequest,
        idempotencyKey: String,
    ): ApiBaseRet<CommandAcceptedData>

    suspend fun resolveApproval(
        sessionId: String,
        approvalId: String,
        request: ApprovalResolutionRequest,
        idempotencyKey: String,
    ): ApiBaseRet<CommandAcceptedData>

    suspend fun replyQuestion(
        sessionId: String,
        questionId: String,
        request: QuestionReplyRequest,
        idempotencyKey: String,
    ): ApiBaseRet<CommandAcceptedData>

    suspend fun cancelTurn(sessionId: String, idempotencyKey: String): ApiBaseRet<CommandAcceptedData>

    suspend fun retryLastTurn(sessionId: String, idempotencyKey: String): ApiBaseRet<CommandAcceptedData>

    suspend fun sync(cursor: String? = null, limit: Int? = null): ApiBaseRet<SyncData>

    /** Emits only server-to-Mobile Session events from `/v1/ws`. */
    fun observeEvents(): Flow<AgentEventEnvelope>
}
