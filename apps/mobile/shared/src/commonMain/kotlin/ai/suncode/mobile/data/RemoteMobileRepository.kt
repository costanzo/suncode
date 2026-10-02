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
import ai.suncode.mobile.remote.protocol.SessionStreamEvent
import ai.suncode.mobile.remote.RemoteHttpException
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.delay
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.Job
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
    private val transportMutex = Mutex()
    private var syncCursor: String? = null
    private var appForeground = true
    private var activeSessionId: String? = null
    private var pollingJob: Job? = null
    private var sessionStreamJob: Job? = null
    private val lastEventIds = mutableMapOf<String, String>()
    private val lastEventSequences = mutableMapOf<String, Long>()
    private var cachePersistJob: Job? = null

    init {
        scope.launch {
            loadCache()
            restartTransport()
        }
    }

    override fun observeHosts(): Flow<List<Host>> = hosts.asStateFlow()

    override fun observeSessions(): Flow<List<Session>> = sessions.asStateFlow()

    override suspend fun sendMessage(sessionId: String, message: String): Result<Unit> = runCatching {
        val session = requireSession(sessionId)
        client.sendMessage(session.hostId, sessionId, SendMessageRequest(message),)
        refreshSession(sessionId)
    }

    override suspend fun answerQuestion(sessionId: String, answer: String): Result<Unit> = runCatching {
        val session = requireSession(sessionId)
        val question = session.pendingQuestion ?: error("No pending question")
        client.replyQuestion(
            session.hostId,
            sessionId,
            question.id,
            ai.suncode.mobile.remote.protocol.QuestionReplyRequest(listOf(answer), question.revision),
        )
        refreshSession(sessionId)
    }

    override suspend fun resolveApproval(sessionId: String, approvalId: String, action: String, expectedRevision: Int): Result<Unit> = runCatching {
        client.resolveApproval(
            session.hostId,
            sessionId,
            approvalId,
            ai.suncode.mobile.remote.protocol.ApprovalResolutionRequest(action, expectedRevision),
        )
        refreshSession(sessionId)
    }

    override suspend fun cancelTurn(sessionId: String): Result<Unit> = runCatching {
        client.cancelTurn(requireSession(sessionId).hostId, sessionId)
        refreshSession(sessionId)
    }

    override suspend fun retryLastTurn(sessionId: String): Result<Unit> = runCatching {
        client.retryLastTurn(requireSession(sessionId).hostId, sessionId)
        refreshSession(sessionId)
    }

    override suspend fun createSession(host: Host, project: Project, title: String, firstMessage: String): Result<String> = runCatching {
        val sessionId = client.createSession(
            host.id,
            CreateSessionRequest(project.id, title.ifBlank { null }, firstMessage.ifBlank { null }),
        ).data?.sessionId ?: error("Create Session response did not include a Session ID")
        refreshSession(sessionId)
        sessionId
    }

    override suspend fun pairHost(pairingPayload: String): Result<Host> = runCatching {
        require(pairingPayload.isNotBlank()) { "Pairing payload is required" }
        val query = parsePairingUrl(pairingPayload)
        val hostId = query["hostId"] ?: error("Pairing URL has no hostId")
        val code = query["code"] ?: error("Pairing URL has no code")
        client.rememberPairingKey(query["k"] ?: error("Pairing URL has no encryption key"))
        val host = client.exchangePairing(
            hostId,
            PairingExchangeRequest(code, "SunCode Mobile"),
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

    override suspend fun openSession(sessionId: String) {
        activeSessionId = sessionId
        restartTransport()
    }

    override suspend fun closeSession(sessionId: String) {
        if (activeSessionId != sessionId) return
        activeSessionId = null
        restartTransport()
    }

    override suspend fun setAppForeground(foreground: Boolean) {
        appForeground = foreground
        restartTransport()
    }

    override suspend fun clearOfflineCache(): Result<Unit> = runCatching {
        cachePersistJob?.cancel()
        projectionMutex.withLock {
            syncCursor = null
            lastEventIds.clear()
            lastEventSequences.clear()
            sessions.value = emptyList()
            hosts.value = emptyList()
            cacheStore.clear()
        }
    }

    suspend fun refresh() {
        val hostId = hosts.value.firstOrNull()?.id ?: return
        val page = client.listSessions(hostId, limit = 100).data
        projectionMutex.withLock {
            val host = hosts.value.firstOrNull()
            sessions.value = page?.items?.map { it.toDomain(host?.id.orEmpty(), host?.name.orEmpty()) } ?: emptyList()
            syncCursor = null
            reconcileSessionHosts()
            persistLocked()
        }
    }

    private suspend fun synchronize() {
        var cursor = syncCursor
        var firstPage = true
        while (true) {
            val hostId = hosts.value.firstOrNull()?.id ?: return
            val data = client.sync(hostId, cursor, 100).data
            if (data == null) {
                if (firstPage) refresh()
                return
            }
            val projectedHosts = listOfNotNull(data.host).map { host ->
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
                    sessions.value = data.sessions.map { it.toDomain(hostId, projectedHosts.firstOrNull()?.name.orEmpty()) }
                    hosts.value = projectedHosts
                } else {
                    val incoming = data.sessions.associateBy { it.id }
                    val existingIds = sessions.value.mapTo(hashSetOf()) { it.id }
                    sessions.value = sessions.value
                        .asSequence()
                        .map { incoming[it.id]?.toDomain(hostId, projectedHosts.firstOrNull()?.name.orEmpty()) ?: it }
                        .plus(incoming.values.filterNot { it.id in existingIds }.map { it.toDomain(hostId, projectedHosts.firstOrNull()?.name.orEmpty()) })
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
        val detail = client.getSession(requireSession(sessionId).hostId, sessionId).data ?: return
        applySessionDetail(detail)
    }

    private suspend fun applySessionDetail(detail: ai.suncode.mobile.remote.protocol.SessionDetailDto) {
        val host = hosts.value.firstOrNull { it.projects.any { project -> project.id == detail.project.id } }
        val updated = detail.toDomain(host?.id.orEmpty(), host?.name.orEmpty())
        projectionMutex.withLock {
            sessions.value = if (sessions.value.any { it.id == updated.id }) {
                sessions.value.map { existing -> if (existing.id == updated.id) updated else existing }
            } else {
                listOf(updated) + sessions.value
            }
            reconcileSessionHosts()
            persistLocked()
        }
    }

    /** Applies event shapes that carry enough information for a lossless local update. */
    private enum class EventReduction { APPLIED, IGNORED, NEEDS_SNAPSHOT }

    private suspend fun reduceEvent(event: AgentEventEnvelope): EventReduction {
        val sessionId = event.sessionId
        return projectionMutex.withLock {
            val existing = sessions.value.firstOrNull { it.id == sessionId } ?: return@withLock EventReduction.NEEDS_SNAPSHOT
            if (event.sequence != null && event.sequence <= (lastEventSequences[sessionId] ?: -1L)) return@withLock EventReduction.IGNORED
            if (event.eventId != null && lastEventIds[sessionId] == event.eventId) return@withLock EventReduction.IGNORED
            val updated = existing.applyRemoteEvent(event) ?: return@withLock EventReduction.IGNORED
            sessions.value = sessions.value.map { if (it.id == sessionId) updated else it }
            reconcileSessionHosts()
            if (event.eventType == RemoteEventTypes.ASSISTANT_DELTA) scheduleCachePersist() else persistLocked()
            EventReduction.APPLIED
        }
    }

    private suspend fun restartTransport() {
        transportMutex.withLock {
            pollingJob?.cancel()
            sessionStreamJob?.cancel()
            pollingJob?.join()
            sessionStreamJob?.join()
            pollingJob = null
            sessionStreamJob = null
            if (!appForeground) return
            val sessionId = activeSessionId
            if (sessionId == null) {
                pollingJob = scope.launch { pollProjection() }
            } else {
                sessionStreamJob = scope.launch { observeSession(sessionId) }
            }
        }
    }

    private suspend fun pollProjection() {
        var retryDelayMs = POLL_INTERVAL_MS
        while (currentCoroutineContext().isActive && appForeground && activeSessionId == null) {
            try {
                synchronize()
                retryDelayMs = POLL_INTERVAL_MS
                delay(POLL_INTERVAL_MS)
            } catch (_: Throwable) {
                currentCoroutineContext().ensureActive()
                delay(retryDelayMs)
                retryDelayMs = min(retryDelayMs * 2, MAX_RETRY_DELAY_MS)
            }
        }
    }

    private suspend fun observeSession(sessionId: String) {
        var retryDelayMs = INITIAL_RETRY_DELAY_MS
        while (currentCoroutineContext().isActive && appForeground && activeSessionId == sessionId) {
            try {
                client.observeSessionEvents(requireSession(sessionId).hostId, sessionId, lastEventIds[sessionId]).collect { frame ->
                    handleSessionStreamEvent(sessionId, frame)
                }
                delay(withJitter(retryDelayMs))
                retryDelayMs = min(retryDelayMs * 2, MAX_RETRY_DELAY_MS)
            } catch (failure: RemoteHttpException) {
                if (failure.status == HttpStatusCode.Gone) {
                    lastEventIds.remove(sessionId)
                    persistEventIds()
                    refreshSession(sessionId)
                    retryDelayMs = INITIAL_RETRY_DELAY_MS
                    continue
                }
                currentCoroutineContext().ensureActive()
                delay(withJitter(retryDelayMs))
                retryDelayMs = min(retryDelayMs * 2, MAX_RETRY_DELAY_MS)
            } catch (_: Throwable) {
                currentCoroutineContext().ensureActive()
                delay(withJitter(retryDelayMs))
                retryDelayMs = min(retryDelayMs * 2, MAX_RETRY_DELAY_MS)
            }
        }
    }

    private suspend fun handleSessionStreamEvent(sessionId: String, frame: SessionStreamEvent) {
        if (frame.eventType == SESSION_SNAPSHOT_EVENT) {
            val snapshot = json.decodeFromString<ai.suncode.mobile.remote.protocol.SessionSnapshotEnvelope>(frame.data)
            if (snapshot.sessionId != sessionId) return
            applySessionDetail(snapshot.snapshot)
            lastEventSequences[sessionId] = snapshot.sequence
            (frame.eventId ?: snapshot.eventId)?.let { lastEventIds[sessionId] = it }
            persistEventIds()
            return
        }
        val envelope = json.decodeFromString<AgentEventEnvelope>(frame.data).let { event ->
            if (event.eventId == null && frame.eventId != null) event.copy(eventId = frame.eventId) else event
        }
        if (envelope.sessionId != sessionId) return
        val reduction = reduceEvent(envelope)
        envelope.sequence?.let { sequence ->
            if (sequence > (lastEventSequences[sessionId] ?: -1L)) lastEventSequences[sessionId] = sequence
        }
        frame.eventId?.let { lastEventIds[sessionId] = it }
        if (envelope.eventType == RemoteEventTypes.ASSISTANT_DELTA) scheduleCachePersist() else persistEventIds()
        if (reduction == EventReduction.NEEDS_SNAPSHOT) refreshSession(sessionId)
    }

    private suspend fun persistEventIds() {
        projectionMutex.withLock { persistLocked() }
    }

    private fun scheduleCachePersist() {
        cachePersistJob?.cancel()
        cachePersistJob = scope.launch {
            delay(CACHE_PERSIST_DEBOUNCE_MS)
            projectionMutex.withLock { persistLocked() }
        }
    }

    private fun withJitter(baseDelayMs: Long): Long =
        (baseDelayMs * 0.8).toLong() + Random.nextLong((baseDelayMs * 0.4).toLong().coerceAtLeast(1L))

    private suspend fun loadCache() {
        val raw = cacheStore.read() ?: return
        try {
            val snapshot = json.decodeFromString<MobileCacheSnapshot>(raw)
            projectionMutex.withLock {
                syncCursor = snapshot.syncCursor
                lastEventIds.putAll(snapshot.sessionEventIds)
                lastEventSequences.putAll(snapshot.sessionEventSequences)
                hosts.value = snapshot.hosts.map { it.toDomain() }
                sessions.value = snapshot.sessions.map { it.toDomain(snapshot.hosts.firstOrNull()?.id.orEmpty(), snapshot.hosts.firstOrNull()?.name.orEmpty()) }
                reconcileSessionHosts()
            }
        } catch (_: Throwable) {
            cacheStore.clear()
        }
    }

    private suspend fun persistLocked() {
        cacheStore.write(MobileCacheSnapshot(
            syncCursor = syncCursor,
            sessionEventIds = lastEventIds.toMap(),
            sessionEventSequences = lastEventSequences.toMap(),
            sessions = sessions.value.map { it.toCached() },
            hosts = hosts.value.map { it.toCached() },
        ).encode(json))
    }

    private fun requireSession(sessionId: String): Session = sessions.value.firstOrNull { it.id == sessionId } ?: error("Unknown session: $sessionId")

    private fun parsePairingUrl(value: String): Map<String, String> {
        val trimmed = value.trim()
        require(trimmed.startsWith("http://") || trimmed.startsWith("https://")) { "Pairing URL must use http or https" }
        val query = trimmed.substringAfter('?', "")
        require(query.isNotBlank()) { "Pairing URL must include a query" }
        val values = query.split('&').mapNotNull { part ->
            val separator = part.indexOf('=')
            if (separator <= 0) return@mapNotNull null
            percentDecode(part.substring(0, separator)) to percentDecode(part.substring(separator + 1))
        }.toMap()
        require(!values["code"].isNullOrBlank() && !values["hostId"].isNullOrBlank() && !values["k"].isNullOrBlank()) {
            "Pairing URL is incomplete"
        }
        return values
    }

    private fun percentDecode(value: String): String {
        val bytes = ByteArray(value.length)
        var count = 0
        var index = 0
        while (index < value.length) {
            if (value[index] == '%' && index + 2 < value.length) {
                val hex = value.substring(index + 1, index + 3).toIntOrNull(16)
                if (hex != null) {
                    bytes[count++] = hex.toByte()
                    index += 3
                    continue
                }
            }
            bytes[count++] = value[index].code.toByte()
            index++
        }
        return bytes.copyOf(count).decodeToString()
    }

    private companion object {
        const val INITIAL_RETRY_DELAY_MS = 1_000L
        const val MAX_RETRY_DELAY_MS = 30_000L
        const val POLL_INTERVAL_MS = 15_000L
        const val CACHE_PERSIST_DEBOUNCE_MS = 400L
        const val PREVIEW_LIMIT = 500
        const val SESSION_SNAPSHOT_EVENT = "session.snapshot"
    }
}

internal fun Session.applyRemoteEvent(event: AgentEventEnvelope): Session? = when (event.eventType) {
    RemoteEventTypes.TURN_STATE -> copy(
        state = event.payload.string("state")?.toSessionState() ?: return null,
        updatedLabel = event.occurredAt,
    )
    RemoteEventTypes.TURN_COMPLETED -> copy(state = SessionState.IDLE, updatedLabel = event.occurredAt, streamingAssistantText = null)
    RemoteEventTypes.ASSISTANT_DELTA -> {
        val text = event.payload.string("text") ?: return null
        if (text.isBlank()) this else copy(
            state = SessionState.RUNNING,
            preview = (streamingAssistantText.orEmpty() + text).takeLast(500),
            updatedLabel = event.occurredAt,
            streamingAssistantText = streamingAssistantText.orEmpty() + text,
        )
    }
    RemoteEventTypes.MESSAGE_USER, RemoteEventTypes.MESSAGE_ASSISTANT -> {
        val messageId = event.payload.string("message_id") ?: return null
        val body = event.payload.messageText() ?: return null
        if (messages.any { it.id == messageId }) this else copy(
            preview = body,
            updatedLabel = event.occurredAt,
            streamingAssistantText = null,
            messages = messages + Message(
                messageId,
                if (event.eventType == RemoteEventTypes.MESSAGE_USER) MessageAuthor.USER else MessageAuthor.AGENT,
                body,
            ),
        )
    }
    RemoteEventTypes.APPROVAL_REQUESTED -> {
        val approvalId = event.payload.string("approval_id") ?: return null
        val operation = event.payload.string("operation") ?: return null
        copy(
            state = SessionState.WAITING_FOR_APPROVAL,
            updatedLabel = event.occurredAt,
            pendingApproval = PendingApproval(approvalId, revision, event.payload.string("risk") ?: "other", operation, event.payload["arguments"]?.toString()),
        )
    }
    RemoteEventTypes.APPROVAL_RESOLVED -> copy(
        pendingApproval = null,
        state = if (state == SessionState.WAITING_FOR_APPROVAL) SessionState.IDLE else state,
        updatedLabel = event.occurredAt,
    )
    RemoteEventTypes.QUESTION_ASKED -> {
        val requestId = event.payload.string("request_id") ?: return null
        val question = event.payload.questionItems().firstOrNull() ?: return null
        copy(
            state = SessionState.WAITING_FOR_ANSWER,
            updatedLabel = event.occurredAt,
            pendingQuestion = PendingQuestion(requestId, revision, question.prompt, question.options, question.allowsFreeText),
        )
    }
    RemoteEventTypes.QUESTION_REPLIED, RemoteEventTypes.QUESTION_REJECTED -> copy(
        pendingQuestion = null,
        state = if (state == SessionState.WAITING_FOR_ANSWER) SessionState.IDLE else state,
        updatedLabel = event.occurredAt,
    )
    else -> null
}

private fun JsonObject.string(name: String): String? = this[name]?.jsonPrimitive?.contentOrNull

private fun JsonObject.messageText(): String? {
    val message = this["message"]?.jsonObject ?: return null
    return message["content"]?.jsonArray
        ?.mapNotNull { it.jsonObject["text"]?.jsonPrimitive?.contentOrNull }
        ?.joinToString("")
        ?.takeIf { it.isNotBlank() }
}

private data class EventQuestion(
    val prompt: String,
    val options: List<String>,
    val allowsFreeText: Boolean,
)

private fun JsonObject.questionItems(): List<EventQuestion> =
    this["questions"]?.jsonArray.orEmpty().mapNotNull { value ->
        val question = value.jsonObject
        val prompt = question["question"]?.jsonPrimitive?.contentOrNull ?: return@mapNotNull null
        val options = question["options"]?.jsonArray.orEmpty().mapNotNull { option ->
            option.jsonObject["label"]?.jsonPrimitive?.contentOrNull
        }
        if (options.isEmpty()) return@mapNotNull null
        EventQuestion(prompt, options, question["custom"]?.jsonPrimitive?.contentOrNull?.toBoolean() == true)
    }

private fun ai.suncode.mobile.remote.protocol.SessionSummaryDto.toDomain(hostId: String, hostName: String) = Session(
    id = id,
    title = title,
    hostId = hostId,
    hostName = hostName,
    projectId = project.id,
    projectName = project.displayName,
    state = state.toSessionState(),
    updatedLabel = updatedAt,
    preview = preview,
    revision = 0,
)

private fun ai.suncode.mobile.remote.protocol.SessionDetailDto.toDomain(hostId: String, hostName: String) = Session(
    id = id,
    title = title,
    hostId = hostId,
    hostName = hostName,
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
    endpoint = "",
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
