package ai.suncode.mobile

import ai.suncode.mobile.remote.protocol.AgentEventEnvelope
import ai.suncode.mobile.remote.protocol.ApiBaseRet
import ai.suncode.mobile.remote.protocol.PairingExchangeData
import ai.suncode.mobile.remote.protocol.SessionDetailDto
import ai.suncode.mobile.remote.protocol.SyncData
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull

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
                  "endpoint": "192.168.1.24",
                  "connectionState": "connected",
                  "projectCount": 2,
                  "activeSessionCount": 1,
                  "lastSeenAt": "2026-09-26T11:59:00Z",
                  "protocolVersion": "remote.v1"
                }
              }
            }
            """.trimIndent(),
        )

        assertEquals("access-1", response.data?.accessToken)
        assertEquals("MacBook Pro", response.data?.host?.displayName)
        assertEquals("remote.v1", response.data?.host?.protocolVersion)
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
                "hosts": [],
                "sessions": [],
                "removedSessionIds": ["session-old"],
                "futureField": "ignored"
              }
            }
            """.trimIndent(),
        )

        assertEquals("cursor-2", response.data?.cursor)
        assertEquals(listOf("session-old"), response.data?.removedSessionIds)
        assertEquals(false, response.data?.hasMore)
    }

    @Test
    fun keepsUnknownWebSocketEventTypesAndPayload() {
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
        assertEquals(true, event.payload.jsonObject["new_field"]?.toString()?.toBoolean())
    }
}
