package ai.suncode.mobile.data

import ai.suncode.mobile.domain.Message
import ai.suncode.mobile.domain.MessageAuthor
import ai.suncode.mobile.domain.PendingApproval
import ai.suncode.mobile.domain.PendingQuestion
import ai.suncode.mobile.domain.Host
import ai.suncode.mobile.domain.HostConnectionState
import ai.suncode.mobile.domain.Project
import ai.suncode.mobile.domain.Session
import ai.suncode.mobile.domain.SessionState
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

/** Small platform boundary for the durable Mobile projection. */
interface MobileCacheStore {
    suspend fun read(): String?
    suspend fun write(value: String)
    suspend fun clear()
}

class InMemoryMobileCacheStore : MobileCacheStore {
    private var value: String? = null

    override suspend fun read(): String? = value
    override suspend fun write(value: String) {
        this.value = value
    }
    override suspend fun clear() {
        value = null
    }
}

@Serializable
internal data class MobileCacheSnapshot(
    val syncCursor: String? = null,
    val sessions: List<CachedSession> = emptyList(),
    val hosts: List<CachedHost> = emptyList(),
)

@Serializable
internal data class CachedHost(
    val id: String,
    val name: String,
    val endpoint: String,
    val state: String,
    val projects: List<CachedProject> = emptyList(),
)

@Serializable
internal data class CachedProject(
    val id: String,
    val name: String,
    val activeSessions: Int,
)

@Serializable
internal data class CachedSession(
    val id: String,
    val title: String,
    val hostId: String,
    val hostName: String,
    val projectId: String,
    val projectName: String,
    val state: String,
    val updatedLabel: String,
    val preview: String,
    val messages: List<CachedMessage> = emptyList(),
    val revision: Int = 0,
    val pendingApproval: CachedApproval? = null,
    val pendingQuestion: CachedQuestion? = null,
)

@Serializable
internal data class CachedMessage(
    val id: String,
    val author: String,
    val body: String,
)

@Serializable
internal data class CachedApproval(
    val id: String,
    val revision: Int,
    val risk: String,
    val summary: String,
    val detail: String? = null,
)

@Serializable
internal data class CachedQuestion(
    val id: String,
    val revision: Int,
    val prompt: String,
    val options: List<String>,
    val allowsFreeText: Boolean = false,
)

internal fun Session.toCached() = CachedSession(
    id = id,
    title = title,
    hostId = hostId,
    hostName = hostName,
    projectId = projectId,
    projectName = projectName,
    state = state.name,
    updatedLabel = updatedLabel,
    preview = preview,
    messages = messages.map { CachedMessage(it.id, it.author.name, it.body) },
    revision = revision,
    pendingApproval = pendingApproval?.let { CachedApproval(it.id, it.revision, it.risk, it.summary, it.detail) },
    pendingQuestion = pendingQuestion?.let { CachedQuestion(it.id, it.revision, it.prompt, it.options, it.allowsFreeText) },
)

internal fun Host.toCached() = CachedHost(
    id = id,
    name = name,
    endpoint = endpoint,
    state = state.name,
    projects = projects.map { CachedProject(it.id, it.name, it.activeSessions) },
)

internal fun CachedSession.toDomain() = Session(
    id = id,
    title = title,
    hostId = hostId,
    hostName = hostName,
    projectId = projectId,
    projectName = projectName,
    state = runCatching { SessionState.valueOf(state) }.getOrDefault(SessionState.IDLE),
    updatedLabel = updatedLabel,
    preview = preview,
    messages = messages.map { Message(it.id, runCatching { MessageAuthor.valueOf(it.author) }.getOrDefault(MessageAuthor.AGENT), it.body) },
    revision = revision,
    pendingApproval = pendingApproval?.let { PendingApproval(it.id, it.revision, it.risk, it.summary, it.detail) },
    pendingQuestion = pendingQuestion?.let { PendingQuestion(it.id, it.revision, it.prompt, it.options, it.allowsFreeText) },
)

internal fun CachedHost.toDomain() = Host(
    id = id,
    name = name,
    endpoint = endpoint,
    state = runCatching { HostConnectionState.valueOf(state) }.getOrDefault(HostConnectionState.OFFLINE),
    projects = projects.map { Project(it.id, it.name, it.activeSessions) },
)

internal fun MobileCacheSnapshot.encode(json: Json): String = json.encodeToString(this)
