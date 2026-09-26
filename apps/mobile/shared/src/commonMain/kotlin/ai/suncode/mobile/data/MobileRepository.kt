package ai.suncode.mobile.data

import ai.suncode.mobile.domain.Host
import ai.suncode.mobile.domain.Message
import ai.suncode.mobile.domain.Project
import ai.suncode.mobile.domain.Session
import kotlinx.coroutines.flow.Flow

interface MobileRepository {
    fun observeHosts(): Flow<List<Host>>
    fun observeSessions(): Flow<List<Session>>
    suspend fun sendMessage(sessionId: String, message: String): Result<Unit>
    suspend fun answerQuestion(sessionId: String, answer: String): Result<Unit>
    suspend fun resolveApproval(sessionId: String, approvalId: String, action: String, expectedRevision: Int): Result<Unit>
    suspend fun cancelTurn(sessionId: String): Result<Unit>
    suspend fun retryLastTurn(sessionId: String): Result<Unit>
    /** Acceptance does not include a Session ID; the Session arrives through the event stream. */
    suspend fun createSession(host: Host, project: Project, title: String, firstMessage: String): Result<Unit>
    suspend fun pairHost(pairingPayload: String): Result<Host>
    suspend fun reconnect(hostId: String): Result<Unit>
    suspend fun clearOfflineCache(): Result<Unit>
}

/** Deterministic preview repository used when no Remote Control base URL is configured. */
class FakeMobileRepository : MobileRepository {
    private val hosts = listOf(
        Host(
            id = "macbook-pro",
            name = "MacBook Pro",
            endpoint = "192.168.1.24",
            state = ai.suncode.mobile.domain.HostConnectionState.CONNECTED,
            projects = listOf(
                Project("suncode", "suncode", 2),
                Project("mobile-app", "mobile-app", 1),
                Project("design-system", "design-system", 0),
            ),
        ),
        Host(
            id = "windows-desktop",
            name = "Windows Desktop",
            endpoint = "10.0.0.18",
            state = ai.suncode.mobile.domain.HostConnectionState.OFFLINE,
            projects = listOf(Project("mobile-app", "mobile-app", 1)),
        ),
    )

    private val sessions = listOf(
        Session(
            id = "login-redirect",
            title = "Fix login redirect",
            hostId = "macbook-pro",
            hostName = "MacBook Pro",
            projectId = "suncode",
            projectName = "suncode",
            state = ai.suncode.mobile.domain.SessionState.WAITING_FOR_APPROVAL,
            updatedLabel = "2 min ago",
            preview = "I found one file write that needs your approval.",
            messages = listOf(
                Message("m1", ai.suncode.mobile.domain.MessageAuthor.USER, "Fix the redirect loop after login and add a regression test."),
                Message("m2", ai.suncode.mobile.domain.MessageAuthor.AGENT, "I found the redirect loop in auth/redirect.ts. The fix is ready, but writing the file requires your approval."),
            ),
            pendingApproval = ai.suncode.mobile.domain.PendingApproval(
                id = "approval-1",
                revision = 1,
                risk = "filesystem",
                summary = "Write 1 file",
                detail = "src/auth/redirect.ts",
            ),
        ),
        Session(
            id = "api-client",
            title = "Refactor API client",
            hostId = "windows-desktop",
            hostName = "Windows Desktop",
            projectId = "mobile-app",
            projectName = "mobile-app",
            state = ai.suncode.mobile.domain.SessionState.RUNNING,
            updatedLabel = "12 min ago",
            preview = "Updating the request retry policy and its tests.",
        ),
        Session(
            id = "release-notes",
            title = "Review release notes",
            hostId = "macbook-pro",
            hostName = "MacBook Pro",
            projectId = "design-system",
            projectName = "design-system",
            state = ai.suncode.mobile.domain.SessionState.IDLE,
            updatedLabel = "Yesterday",
            preview = "The latest response is ready to review.",
        ),
    )

    override fun observeHosts(): Flow<List<Host>> = kotlinx.coroutines.flow.flowOf(hosts)
    override fun observeSessions(): Flow<List<Session>> = kotlinx.coroutines.flow.flowOf(sessions)
    override suspend fun sendMessage(sessionId: String, message: String): Result<Unit> = Result.success(Unit)
    override suspend fun answerQuestion(sessionId: String, answer: String): Result<Unit> = Result.success(Unit)
    override suspend fun resolveApproval(sessionId: String, approvalId: String, action: String, expectedRevision: Int): Result<Unit> = Result.success(Unit)
    override suspend fun cancelTurn(sessionId: String): Result<Unit> = Result.success(Unit)
    override suspend fun retryLastTurn(sessionId: String): Result<Unit> = Result.success(Unit)
    override suspend fun createSession(host: Host, project: Project, title: String, firstMessage: String): Result<Unit> = Result.success(Unit)
    override suspend fun pairHost(pairingPayload: String): Result<Host> = Result.failure(NotImplementedError("Remote pairing is not connected yet"))
    override suspend fun reconnect(hostId: String): Result<Unit> = Result.failure(NotImplementedError("Remote reconnect is not connected yet"))
    override suspend fun clearOfflineCache(): Result<Unit> = Result.success(Unit)
}
