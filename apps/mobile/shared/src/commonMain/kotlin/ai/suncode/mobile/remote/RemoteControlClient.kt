package ai.suncode.mobile.remote

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
import ai.suncode.mobile.remote.protocol.SessionStreamEvent
import ai.suncode.mobile.remote.protocol.SyncData
import ai.suncode.mobile.remote.protocol.TokenData
import kotlinx.coroutines.flow.Flow

/**
 * Typed boundary for the Mobile-facing Remote Control API.
 *
 * Implementations own HTTP authentication, request headers, response status handling,
 * and Session SSE lifecycle. The UI and domain layer never construct protocol URLs or
 * envelopes directly.
 */
interface RemoteControlClient {
    suspend fun rememberPairingKey(key: String) = Unit
    suspend fun health(): ApiBaseRet<HealthData>

    suspend fun exchangePairing(hostId: String, request: PairingExchangeRequest): ApiBaseRet<PairingExchangeData>

    suspend fun refreshToken(request: RefreshTokenRequest): ApiBaseRet<TokenData>

    suspend fun logout()

    suspend fun getHost(hostId: String): ApiBaseRet<HostDto>

    suspend fun listProjects(hostId: String): ApiBaseRet<ProjectsData>

    suspend fun listSessions(
        hostId: String,
        projectId: String? = null,
        cursor: String? = null,
        limit: Int? = null,
    ): ApiBaseRet<SessionPageData>

    suspend fun createSession(hostId: String, request: CreateSessionRequest): ApiBaseRet<CommandAcceptedData>

    suspend fun getSession(hostId: String, sessionId: String): ApiBaseRet<SessionDetailDto>

    suspend fun sendMessage(
        hostId: String,
        sessionId: String,
        request: SendMessageRequest,
    ): ApiBaseRet<CommandAcceptedData>

    suspend fun resolveApproval(
        hostId: String,
        sessionId: String,
        approvalId: String,
        request: ApprovalResolutionRequest,
    ): ApiBaseRet<CommandAcceptedData>

    suspend fun replyQuestion(
        hostId: String,
        sessionId: String,
        questionId: String,
        request: QuestionReplyRequest,
    ): ApiBaseRet<CommandAcceptedData>

    suspend fun cancelTurn(hostId: String, sessionId: String): ApiBaseRet<CommandAcceptedData>

    suspend fun retryLastTurn(hostId: String, sessionId: String): ApiBaseRet<CommandAcceptedData>

    suspend fun sync(hostId: String, cursor: String? = null, limit: Int? = null): ApiBaseRet<SyncData>

    /** Emits SSE frames for one Session, including the initial `session.snapshot` frame. */
    fun observeSessionEvents(hostId: String, sessionId: String, lastEventId: String? = null): Flow<SessionStreamEvent>
}
