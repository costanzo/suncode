package ai.suncode.mobile

import ai.suncode.mobile.remote.protocol.AgentEventEnvelope
import ai.suncode.mobile.remote.protocol.ApiBaseRet
import ai.suncode.mobile.remote.protocol.PairingExchangeData
import ai.suncode.mobile.remote.protocol.SessionDetailDto
import ai.suncode.mobile.remote.protocol.SessionSnapshotEnvelope
import ai.suncode.mobile.remote.protocol.SyncData
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull

class RemoteControlProtocolTest {
    private val json = Json { ignoreUnknownKeys = true }

    @Test
    fun decodesPairingEnvelopeAndPreservesHostProjection() {
        val response = json.decodeFromString<ApiBaseRet<PairingExchangeData>>(
            """
            {
              "code": 0,
              "data": {
                "accessToken": "access-1",
                "refreshToken": "refresh-1",
                "accessTokenExpiresAt": "2026-09-26T12:00:00Z",
                "host": {
                  "id": "host-1",
                  "displayName": "MacBook Pro",
                  "connectionState": "connected",
                  "lastSeenAt": "2026-09-26T11:59:00Z",
                  "agentVersion": "0.1.0"
                }
              }
            }
            """.trimIndent(),
        )

        assertEquals("access-1", response.data?.accessToken)
        assertEquals("MacBook Pro", response.data?.host?.displayName)
        assertEquals("0.1.0", response.data?.host?.agentVersion)
    }

    @Test
    fun decodesSessionDetailWithPendingApproval() {
        val response = json.decodeFromString<ApiBaseRet<SessionDetailDto>>(
            """
            {
              "code": 0,
              "data": {
                "id": "session-1",
                "title": "Fix login redirect",
                "kind": "primary",
                "host": {"id": "host-1", "displayName": "MacBook Pro"},
                "project": {"id": "project-1", "displayName": "suncode"},
                "state": "waiting_for_approval",
                "updatedAt": "2026-09-26T11:59:00Z",
                "preview": "Needs approval",
                "revision": 4,
                "archived": false,
                "messages": [{"id":"message-1","role":"assistant","text":"Please approve","createdAt":"2026-09-26T11:58:00Z"}],
                "pendingApproval": {"id":"approval-1","revision":4,"risk":"filesystem","summary":"Write one file","createdAt":"2026-09-26T11:59:00Z"}
              }
            }
            """.trimIndent(),
        )

        val session = assertNotNull(response.data)
        assertEquals(4, session.revision)
        assertEquals("approval-1", session.pendingApproval?.id)
        assertEquals("assistant", session.messages.single().role)
    }

    @Test
    fun decodesSyncAndIgnoresAdditiveFields() {
        val response = json.decodeFromString<ApiBaseRet<SyncData>>(
            """
            {
              "code": 0,
              "data": {
                "cursor": "cursor-2",
                "resetRequired": false,
                "host": {"id":"host-1","displayName":"MacBook Pro","connectionState":"connected","lastSeenAt":null,"agentVersion":"0.1.0"},
                "sessions": [],
                "futureField": "ignored"
              }
            }
            """.trimIndent(),
        )

        assertEquals("cursor-2", response.data?.cursor)
        assertEquals("host-1", response.data?.host?.id)
        assertEquals(false, response.data?.hasMore)
    }

    @Test
    fun decodesSessionEventCursorAndSnapshotEnvelope() {
        val snapshot = json.decodeFromString<SessionSnapshotEnvelope>(
            """
            {
              "event_id": "session-1:12",
              "session_id": "session-1",
              "sequence": 12,
              "session_revision": 4,
              "snapshot": {
                "id": "session-1",
                "title": "Fix login redirect",
                "kind": "primary",
                "project": {"id":"project-1","displayName":"suncode"},
                "state": "running",
                "updatedAt": "2026-09-26T12:00:00Z",
                "preview": "Working",
                "revision": 4,
                "archived": false,
                "messages": []
              }
            }
            """.trimIndent(),
        )

        assertEquals("session-1:12", snapshot.eventId)
        assertEquals(12, snapshot.sequence)
        assertEquals("session-1", snapshot.snapshot.id)

        val event = json.decodeFromString<AgentEventEnvelope>(
            """{"session_id":"session-1","occurred_at":"now","event_type":"future.event","event_id":"session-1:13","sequence":13,"session_revision":5,"payload":{}}""",
        )
        assertEquals(13, event.sequence)
        assertEquals("session-1:13", event.eventId)
    }

    @Test
    fun keepsUnknownSseEventTypesAndPayload() {
        val event = json.decodeFromString<AgentEventEnvelope>(
            """
            {
              "session_id": "session-1",
              "occurred_at": "2026-09-26T12:00:00Z",
              "event_type": "future.event",
              "payload": {"new_field": true}
            }
            """.trimIndent(),
        )

        assertEquals("future.event", event.eventType)
        assertNull(event.sequence)
        assertEquals(true, event.payload.jsonObject["new_field"]?.toString()?.toBoolean())
    }
}
