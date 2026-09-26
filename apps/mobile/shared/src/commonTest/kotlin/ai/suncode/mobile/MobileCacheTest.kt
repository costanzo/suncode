package ai.suncode.mobile

import ai.suncode.mobile.data.InMemoryMobileCacheStore
import ai.suncode.mobile.data.MobileCacheSnapshot
import ai.suncode.mobile.data.toCached
import ai.suncode.mobile.data.toDomain
import ai.suncode.mobile.domain.Message
import ai.suncode.mobile.domain.MessageAuthor
import ai.suncode.mobile.domain.PendingApproval
import ai.suncode.mobile.domain.Host
import ai.suncode.mobile.domain.HostConnectionState
import ai.suncode.mobile.domain.Project
import ai.suncode.mobile.domain.Session
import ai.suncode.mobile.domain.SessionState
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.Json
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class MobileCacheTest {
    private val json = Json { ignoreUnknownKeys = true; explicitNulls = false }

    @Test
    fun snapshotRoundTripsSessionProjection() {
        val session = Session(
            id = "session-1",
            title = "Review release",
            hostId = "host-1",
            hostName = "MacBook Pro",
            projectId = "project-1",
            projectName = "suncode",
            state = SessionState.WAITING_FOR_APPROVAL,
            updatedLabel = "2026-09-26T12:00:00Z",
            preview = "Needs approval",
            messages = listOf(Message("message-1", MessageAuthor.AGENT, "Please approve")),
            revision = 4,
            pendingApproval = PendingApproval("approval-1", 4, "filesystem", "Write one file", "src/main.kt"),
        )
        val snapshot = MobileCacheSnapshot("cursor-2", listOf(session.toCached()))
        val restored = json.decodeFromString<MobileCacheSnapshot>(json.encodeToString(snapshot)).sessions.single().toDomain()

        assertEquals(session, restored)
    }

    @Test
    fun cacheStoreClearsValue() = runBlocking {
        val store = InMemoryMobileCacheStore()
        store.write("snapshot")
        assertEquals("snapshot", store.read())
        store.clear()
        assertNull(store.read())
    }

    @Test
    fun snapshotRoundTripsHostProjects() {
        val host = Host(
            id = "host-1",
            name = "MacBook Pro",
            endpoint = "relay.example",
            state = HostConnectionState.DEGRADED,
            projects = listOf(Project("project-1", "suncode", 2)),
        )
        val snapshot = MobileCacheSnapshot(hosts = listOf(host.toCached()))
        val restored = json.decodeFromString<MobileCacheSnapshot>(json.encodeToString(snapshot)).hosts.single().toDomain()

        assertEquals(host, restored)
    }
}
