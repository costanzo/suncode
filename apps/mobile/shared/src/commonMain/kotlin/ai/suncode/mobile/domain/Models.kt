package ai.suncode.mobile.domain

enum class AppTab { SESSIONS, HOSTS, SETTINGS }

enum class ThemePreference { SYSTEM, LIGHT, DARK }

enum class HostConnectionState { CONNECTED, CONNECTING, OFFLINE, UNAUTHORIZED }

enum class SessionState { IDLE, RUNNING, WAITING_FOR_APPROVAL, WAITING_FOR_ANSWER, FAILED }

data class Host(
    val id: String,
    val name: String,
    val endpoint: String,
    val state: HostConnectionState,
    val projects: List<Project>,
)

data class Project(
    val id: String,
    val name: String,
    val activeSessions: Int,
)

data class Session(
    val id: String,
    val title: String,
    val hostId: String,
    val hostName: String,
    val projectId: String,
    val projectName: String,
    val state: SessionState,
    val updatedLabel: String,
    val preview: String,
    val messages: List<Message> = emptyList(),
)

data class Message(
    val id: String,
    val author: MessageAuthor,
    val body: String,
)

enum class MessageAuthor { USER, AGENT }

data class PendingQuestion(
    val prompt: String,
    val options: List<String>,
)

data class PairingCandidate(
    val hostName: String,
    val endpoint: String,
    val relay: String,
    val fingerprint: String,
)
