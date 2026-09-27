package ai.suncode.mobile

import ai.suncode.mobile.data.applyRemoteEvent
import ai.suncode.mobile.domain.Message
import ai.suncode.mobile.domain.MessageAuthor
import ai.suncode.mobile.domain.Session
import ai.suncode.mobile.domain.SessionState
import ai.suncode.mobile.remote.protocol.AgentEventEnvelope
import kotlinx.serialization.json.Json
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull

class RemoteEventReducerTest {
    private val json = Json { ignoreUnknownKeys = true }

    private val session = Session(
        id = "session-1",
        title = "Implement feature",
        hostId = "host-1",
        hostName = "MacBook Pro",
        projectId = "project-1",
        projectName = "suncode",
        state = SessionState.RUNNING,
        updatedLabel = "before",
        preview = "before",
        messages = listOf(Message("message-1", MessageAuthor.USER, "Start")),
        revision = 7,
    )

    @Test
    fun approvalRequestedCreatesPendingApproval() {
        val event = event(
            "approval.requested",
            """
            {"turn_id":"turn-1","tool_call_id":"call-1","approval_id":"approval-1","operation":"write_file","arguments":{"path":"src/main.kt"}}
            """,
        )

        val updated = assertNotNull(session.applyRemoteEvent(event))
        assertEquals(SessionState.WAITING_FOR_APPROVAL, updated.state)
        assertEquals("approval-1", updated.pendingApproval?.id)
        assertEquals("write_file", updated.pendingApproval?.summary)
        assertEquals("{\"path\":\"src/main.kt\"}", updated.pendingApproval?.detail)
    }

    @Test
    fun questionAskedCreatesPendingQuestionFromToolArguments() {
        val event = event(
            "question.asked",
            """
            {"request_id":"request-1","turn_id":"turn-1","tool_call_id":"call-1","questions":[{"question":"Choose a mode","header":"Mode","options":[{"label":"Fast","description":"Quick"},{"label":"Safe","description":"Careful"}],"custom":true}]}
            """,
        )

        val updated = assertNotNull(session.applyRemoteEvent(event))
        assertEquals(SessionState.WAITING_FOR_ANSWER, updated.state)
        assertEquals("request-1", updated.pendingQuestion?.id)
        assertEquals("Choose a mode", updated.pendingQuestion?.prompt)
        assertEquals(listOf("Fast", "Safe"), updated.pendingQuestion?.options)
        assertEquals(true, updated.pendingQuestion?.allowsFreeText)
    }

    @Test
    fun assistantDeltasAccumulateAndFinalMessageClearsStreamingText() {
        val first = assertNotNull(session.applyRemoteEvent(event("assistant.delta", """{"turn_id":"turn-1","text":"Hello"}""")))
        val second = assertNotNull(first.applyRemoteEvent(event("assistant.delta", """{"turn_id":"turn-1","text":" world"}""")))
        assertEquals("Hello world", second.streamingAssistantText)
        assertEquals("Hello world", second.preview)

        val completed = assertNotNull(second.applyRemoteEvent(event(
            "message.assistant",
            """{"message_id":"message-2","turn_id":"turn-1","message":{"role":"assistant","content":[{"type":"text","text":"Hello world"}]}}""",
        )))
        assertEquals(null, completed.streamingAssistantText)
        assertEquals("Hello world", completed.messages.last().body)
    }

    private fun event(type: String, payload: String): AgentEventEnvelope = json.decodeFromString(
        """{"session_id":"session-1","occurred_at":"2026-09-26T12:00:00Z","event_type":"$type","payload":$payload}""",
    )
}
