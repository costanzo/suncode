package ai.suncode.mobile.data

import ai.suncode.mobile.domain.Host
import ai.suncode.mobile.domain.HostConnectionState
import ai.suncode.mobile.domain.Message
import ai.suncode.mobile.domain.MessageAuthor
import ai.suncode.mobile.domain.PendingApproval
import ai.suncode.mobile.domain.PendingQuestion
import ai.suncode.mobile.domain.Project
import ai.suncode.mobile.domain.Session
import ai.suncode.mobile.domain.SessionState
import ai.suncode.mobile.remote.RemoteControlClient
import ai.suncode.mobile.remote.protocol.CreateSessionRequest
import ai.suncode.mobile.remote.protocol.PairingExchangeRequest
import ai.suncode.mobile.remote.protocol.SendMessageRequest
import ai.suncode.mobile.remote.protocol.AgentEventEnvelope
import ai.suncode.mobile.remote.protocol.RemoteEventTypes
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.delay
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.contentOrNull
import kotlin.math.min
import kotlin.random.Random

/** Repository adapter that projects Remote Control DTOs into the Mobile domain models. */
class RemoteMobileRepository(
    private val client: RemoteControlClient,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.Default),
    private val cacheStore: MobileCacheStore = InMemoryMobileCacheStore(),
    private val json: Json = Json { ignoreUnknownKeys = true; explicitNulls = false },
) : MobileRepository {
    private val hosts = MutableStateFlow<List<Host>>(emptyList())
    private val sessions = MutableStateFlow<List<Session>>(emptyList())
    private val projectionMutex = Mutex()
    private var syncCursor: String? = null

    init {
        scope.launch {
            loadCache()
            var retryDelayMs = INITIAL_RETRY_DELAY_MS
            while (currentCoroutineContext().isActive) {
                try {
                    synchronize()
                    client.observeEvents().collect { event ->
                        if (event.sessionId.isNotBlank() && !reduceEvent(event)) refreshSession(event.sessionId)
                    }
                    retryDelayMs = INITIAL_RETRY_DELAY_MS
                } catch (_: Throwable) {
                    currentCoroutineContext().ensureActive()
                    delay(retryDelayMs)
                    retryDelayMs = min(retryDelayMs * 2, MAX_RETRY_DELAY_MS)
                }
            }
        }
    }

    override fun observeHosts(): Flow<List<Host>> = hosts.asStateFlow()

    override fun observeSessions(): Flow<List<Session>> = sessions.asStateFlow()

    override suspend fun sendMessage(sessionId: String, message: String): Result<Unit> = runCatching {
        client.sendMessage(sessionId, SendMessageRequest(message), key())
        refreshSession(sessionId)
    }

    override suspend fun answerQuestion(sessionId: String, answer: String): Result<Unit> = runCatching {
        val session = requireSession(sessionId)
        val question = session.pendingQuestion ?: error("No pending question")
        client.replyQuestion(
            sessionId,
            question.id,
            ai.suncode.mobile.remote.protocol.QuestionReplyRequest(listOf(answer), question.revision),
            key(),
        )
        refreshSession(sessionId)
    }

    override suspend fun resolveApproval(sessionId: String, approvalId: String, action: String, expectedRevision: Int): Result<Unit> = runCatching {
        client.resolveApproval(
            sessionId,
            approvalId,
            ai.suncode.mobile.remote.protocol.ApprovalResolutionRequest(action, expectedRevision),
            key(),
        )
        refreshSession(sessionId)
    }

    override suspend fun cancelTurn(sessionId: String): Result<Unit> = runCatching {
        client.cancelTurn(sessionId, key())
        refreshSession(sessionId)
    }

    override suspend fun retryLastTurn(sessionId: String): Result<Unit> = runCatching {
        client.retryLastTurn(sessionId, key())
        refreshSession(sessionId)
    }

    override suspend fun createSession(host: Host, project: Project, title: String, firstMessage: String): Result<Unit> = runCatching {
        client.createSession(CreateSessionRequest(host.id, project.id, title.ifBlank { null }, firstMessage.ifBlank { null }), key())
    }

    override suspend fun pairHost(pairingPayload: String): Result<Host> = runCatching {
        require(pairingPayload.isNotBlank()) { "Pairing payload is required" }
        val host = client.exchangePairing(
            PairingExchangeRequest(pairingPayload.trim(), "SunCode Mobile"),
        ).data?.host ?: error("Pairing response did not include a Host")
        val pairedHost = host.toDomain().copy(
            projects = runCatching { client.listProjects(host.id).data?.items.orEmpty() }
                .getOrDefault(emptyList())
                .map { project -> Project(project.id, project.displayName, project.activeSessionCount) },
        )
        projectionMutex.withLock {
            hosts.value = hosts.value.filterNot { it.id == pairedHost.id } + pairedHost
            persistLocked()
        }
        pairedHost
    }

    override suspend fun reconnect(hostId: String): Result<Unit> = runCatching { synchronize() }

    override suspend fun clearOfflineCache(): Result<Unit> = runCatching {
        projectionMutex.withLock {
            syncCursor = null
            sessions.value = emptyList()
            hosts.value = emptyList()
            cacheStore.clear()
        }
    }

    suspend fun refresh() {
        val page = client.listSessions(limit = 100).data
        projectionMutex.withLock {
            sessions.value = page?.items?.map { it.toDomain() } ?: emptyList()
            syncCursor = null
            reconcileSessionHosts()
            persistLocked()
        }
    }

    private suspend fun synchronize() {
        var cursor = syncCursor
        var firstPage = true
        while (true) {
            val data = client.sync(cursor, 100).data
            if (data == null) {
                if (firstPage) refresh()
                return
            }
            val projectedHosts = data.hosts.map { host ->
                val current = hosts.value.firstOrNull { it.id == host.id }
                if (current != null && current.projects.isNotEmpty()) {
                    host.toDomain().copy(projects = current.projects)
                } else {
                    runCatching {
                        host.toDomain().copy(
                            projects = client.listProjects(host.id).data?.items.orEmpty().map { project ->
                                Project(project.id, project.displayName, project.activeSessionCount)
                            },
                        )
                    }.getOrElse { current ?: host.toDomain() }
                }
            }
            projectionMutex.withLock {
                if (firstPage && (cursor == null || data.resetRequired)) {
                    sessions.value = data.sessions.map { it.toDomain() }
                    hosts.value = projectedHosts
                } else {
                    val removed = data.removedSessionIds.toSet()
                    val incoming = data.sessions.associateBy { it.id }
                    val existingIds = sessions.value.mapTo(hashSetOf()) { it.id }
                    sessions.value = sessions.value
                        .asSequence()
                        .filterNot { it.id in removed }
                        .map { incoming[it.id]?.toDomain() ?: it }
                        .plus(incoming.values.filterNot { it.id in existingIds }.map { it.toDomain() })
                        .toList()
                    val incomingHosts = projectedHosts.associateBy { it.id }
                    hosts.value = hosts.value.map { existing ->
                        incomingHosts[existing.id]?.mergeProjects(existing.projects) ?: existing
                    }.plus(incomingHosts.values.filterNot { incoming -> hosts.value.any { it.id == incoming.id } })
                }
                cursor = data.cursor
                syncCursor = cursor
                reconcileSessionHosts()
                persistLocked()
            }
            if (!data.hasMore) break
            firstPage = false
        }
    }

    private fun reconcileSessionHosts() {
        val sessionHosts = sessions.value.groupBy { it.hostId }
        hosts.value = hosts.value.map { existing ->
            val items = sessionHosts[existing.id].orEmpty()
            if (items.isEmpty()) existing else existing.copy(
                projects = mergeProjects(existing.projects, items.map { Project(it.projectId, it.projectName, 0) }),
            )
        }.plus(sessionHosts.filterKeys { hostId -> hosts.value.none { it.id == hostId } }.map { (hostId, items) ->
            Host(hostId, items.first().hostName, "", HostConnectionState.CONNECTED, items.map { Project(it.projectId, it.projectName, 0) }.distinctBy { it.id })
        })
    }

    private suspend fun refreshSession(sessionId: String) {
        val detail = client.getSession(sessionId).data ?: return
        val updated = detail.toDomain()
        projectionMutex.withLock {
            sessions.value = if (sessions.value.any { it.id == sessionId }) {
                sessions.value.map { existing -> if (existing.id == sessionId) updated else existing }
            } else {
                listOf(updated) + sessions.value
            }
            reconcileSessionHosts()
            persistLocked()
        }
    }

    /** Applies event shapes that carry enough information for a lossless local update. */
    private suspend fun reduceEvent(event: AgentEventEnvelope): Boolean {
        val sessionId = event.sessionId
        return projectionMutex.withLock {
            val existing = sessions.value.firstOrNull { it.id == sessionId } ?: return@withLock false
            val updated = when (event.eventType) {
                RemoteEventTypes.TURN_STATE -> existing.copy(
                    state = event.payload.string("state")?.toSessionState() ?: return@withLock false,
                    updatedLabel = event.occurredAt,
                )
                RemoteEventTypes.TURN_COMPLETED -> existing.copy(state = SessionState.IDLE, updatedLabel = event.occurredAt)
                RemoteEventTypes.MESSAGE_USER, RemoteEventTypes.MESSAGE_ASSISTANT -> {
                    val messageId = event.payload.string("message_id") ?: return@withLock false
                    val body = event.payload.messageText() ?: return@withLock false
                    if (existing.messages.any { it.id == messageId }) existing
                    else existing.copy(
                        preview = body,
                        updatedLabel = event.occurredAt,
                        messages = existing.messages + Message(
                            messageId,
                            if (event.eventType == RemoteEventTypes.MESSAGE_USER) MessageAuthor.USER else MessageAuthor.AGENT,
                            body,
                        ),
                    )
                }
                RemoteEventTypes.APPROVAL_RESOLVED -> existing.copy(
                    pendingApproval = null,
                    state = if (existing.state == SessionState.WAITING_FOR_APPROVAL) SessionState.IDLE else existing.state,
                    updatedLabel = event.occurredAt,
                )
                RemoteEventTypes.QUESTION_REPLIED, RemoteEventTypes.QUESTION_REJECTED -> existing.copy(
                    pendingQuestion = null,
                    state = if (existing.state == SessionState.WAITING_FOR_ANSWER) SessionState.IDLE else existing.state,
                    updatedLabel = event.occurredAt,
                )
                else -> return@withLock false
            }
            sessions.value = sessions.value.map { if (it.id == sessionId) updated else it }
            reconcileSessionHosts()
            persistLocked()
            true
        }
    }

    private suspend fun loadCache() {
        val raw = cacheStore.read() ?: return
        try {
            val snapshot = json.decodeFromString<MobileCacheSnapshot>(raw)
            projectionMutex.withLock {
                syncCursor = snapshot.syncCursor
                hosts.value = snapshot.hosts.map { it.toDomain() }
                sessions.value = snapshot.sessions.map { it.toDomain() }
                reconcileSessionHosts()
            }
        } catch (_: Throwable) {
            cacheStore.clear()
        }
    }

    private suspend fun persistLocked() {
        cacheStore.write(MobileCacheSnapshot(
            syncCursor = syncCursor,
            sessions = sessions.value.map { it.toCached() },
            hosts = hosts.value.map { it.toCached() },
        ).encode(json))
    }

    private fun requireSession(sessionId: String): Session = sessions.value.firstOrNull { it.id == sessionId } ?: error("Unknown session: $sessionId")

    private fun key(): String = "mobile-${Random.nextLong().toString(16)}-${Random.nextLong().toString(16)}"

    private companion object {
        const val INITIAL_RETRY_DELAY_MS = 1_000L
        const val MAX_RETRY_DELAY_MS = 30_000L
    }
}

private fun JsonObject.string(name: String): String? = this[name]?.jsonPrimitive?.contentOrNull

private fun JsonObject.messageText(): String? {
    val message = this["message"]?.jsonObject ?: return null
    return message["content"]?.jsonArray
        ?.mapNotNull { it.jsonObject["text"]?.jsonPrimitive?.contentOrNull }
        ?.joinToString("")
        ?.takeIf { it.isNotBlank() }
}

private fun ai.suncode.mobile.remote.protocol.SessionSummaryDto.toDomain() = Session(
    id = id,
    title = title,
    hostId = host.id,
    hostName = host.displayName,
    projectId = project.id,
    projectName = project.displayName,
    state = state.toSessionState(),
    updatedLabel = updatedAt,
    preview = preview,
    revision = 0,
)

private fun ai.suncode.mobile.remote.protocol.SessionDetailDto.toDomain() = Session(
    id = id,
    title = title,
    hostId = host.id,
    hostName = host.displayName,
    projectId = project.id,
    projectName = project.displayName,
    state = state.toSessionState(),
    updatedLabel = updatedAt,
    preview = preview,
    messages = messages.map { Message(it.id, if (it.role == "user") MessageAuthor.USER else MessageAuthor.AGENT, it.text) },
    revision = revision,
    pendingApproval = pendingApproval?.let { PendingApproval(it.id, it.revision, it.risk, it.summary, it.detail) },
    pendingQuestion = pendingQuestion?.let { PendingQuestion(it.id, it.revision, it.prompt, it.options, it.allowsFreeText) },
)

private fun ai.suncode.mobile.remote.protocol.HostDto.toDomain() = Host(
    id = id,
    name = displayName,
    endpoint = endpoint,
    state = when (connectionState) {
        "connected" -> HostConnectionState.CONNECTED
        "connecting" -> HostConnectionState.CONNECTING
        "degraded" -> HostConnectionState.DEGRADED
        "unauthorized" -> HostConnectionState.UNAUTHORIZED
        "incompatible" -> HostConnectionState.INCOMPATIBLE
        else -> HostConnectionState.OFFLINE
    },
    projects = emptyList(),
)

private fun Host.mergeProjects(incoming: List<Project>): Host = copy(
    projects = mergeProjects(projects, incoming),
)

private fun mergeProjects(existing: List<Project>, incoming: List<Project>): List<Project> =
    (existing + incoming).associateBy { it.id }.values.toList()

private fun String.toSessionState() = when (this) {
    "running" -> SessionState.RUNNING
    "waiting_for_approval" -> SessionState.WAITING_FOR_APPROVAL
    "waiting_for_answer" -> SessionState.WAITING_FOR_ANSWER
    "failed" -> SessionState.FAILED
    else -> SessionState.IDLE
}
