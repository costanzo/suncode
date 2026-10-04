package ai.suncode.common.utils;

import ai.suncode.message.remote.DesktopCommand;
import ai.suncode.message.remote.DesktopCommandPayload;
import ai.suncode.message.remote.DesktopEvent;
import ai.suncode.message.remote.DesktopResponse;
import ai.suncode.message.remote.EventPayload;
import ai.suncode.message.remote.MobileEvent;
import ai.suncode.message.remote.SessionSnapshot;
import ai.suncode.message.remote.ApprovalResolutionRequest;
import ai.suncode.message.remote.CommandAcceptedData;
import org.junit.jupiter.api.Test;

import java.time.Instant;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

class MarshallingUtilsTest {
    @Test
    void serializesEventTimeAndConvertsRecordToTree() {
        String occurredAt = "2026-09-27T10:00:00Z";
        EventPayload payload = new EventPayload();
        payload.setTurnId("turn-1");
        payload.setState("running");
        MobileEvent event = new MobileEvent(
                "host:1", 1, "host", "session", "request", "turn.state", occurredAt,
                payload);

        String json = MarshallingUtils.toJson(event);
        MobileEvent restored = MarshallingUtils.fromJson(json, MobileEvent.class);
        assertEquals(occurredAt, restored.occurredAt());
        assertEquals("running", restored.payload().getState());
        assertEquals("turn-1", restored.payload().getTurnId());
        assertNull(restored.payload().getApprovalId());
        assertTrue(json.contains("\"turn_id\":\"turn-1\""));
        assertFalse(json.contains("approval_id"));
        assertEquals("2026-09-27T10:00:00Z", MarshallingUtils.convertValue(event, Map.class).get("occurred_at"));

        DesktopCommand command = new DesktopCommand("request", "host", "session", "session.get", new DesktopCommandPayload());
        assertEquals("session.get", MarshallingUtils.convertValue(command, Map.class).get("command"));
    }

    @Test
    void acceptsTheTwoToolCallsShapes() {
        EventPayload count = MarshallingUtils.fromJson("{\"tool_calls\":3}", EventPayload.class);
        EventPayload calls = MarshallingUtils.fromJson("{\"tool_calls\":[{\"call_id\":\"call-1\"}]}", EventPayload.class);

        assertEquals(3, count.getToolCalls());
        assertTrue(calls.getToolCalls() instanceof java.util.List<?>);
    }

    @Test
    void readsDesktopEventAndNestedPayloadFromSnakeCase() {
        DesktopEvent event = MarshallingUtils.fromJson("""
                {"host_id":"host","session_id":"session","event_type":"message.assistant",
                 "occurred_at":"2026-09-27T10:00:00Z",
                 "payload":{"turn_id":"turn-1","message":{"role":"assistant",
                 "content":[{"type":"text","text":"Hello"}]}}}
                """, DesktopEvent.class);

        assertEquals("message.assistant", event.eventType());
        assertEquals("turn-1", event.payload().getTurnId());
        assertEquals("Hello", event.payload().getMessage().content().get(0).text());
    }

    @Test
    void preservesDesktopCommandFieldsAndConvertsResponseData() {
        DesktopCommandPayload payload = DesktopCommandPayload.resolveApproval(
                "approval-1", new ApprovalResolutionRequest("approve", 7));
        String commandJson = MarshallingUtils.toJson(new DesktopCommand(
                "request-1", "host-1", "session-1", "approval.resolve", payload));

        assertTrue(commandJson.contains("\"approvalId\":\"approval-1\""));
        assertTrue(commandJson.contains("\"expectedRevision\":7"));
        assertFalse(commandJson.contains("\"questionId\""));

        CommandAcceptedData result = MarshallingUtils.convertValue(
                Map.of("requestId", "request-1", "sessionId", "session-1"), CommandAcceptedData.class);
        assertEquals("session-1", result.sessionId());
    }

    @Test
    void keepsOpenResponseAsRawJsonWhileWritingObjectShapedSnapshot() {
        // The controller builds DesktopResponse from the raw /v1/desktop/responses body; it is never deserialized.
        DesktopResponse response = new DesktopResponse(
                "request-1", "host-1", "{\"sessionId\":\"session-1\",\"nested\":{\"count\":2}}");

        assertEquals("{\"sessionId\":\"session-1\",\"nested\":{\"count\":2}}", response.payload());
        assertEquals("session-1", MarshallingUtils.fromJson(response.payload(), CommandAcceptedData.class).sessionId());

        String snapshot = MarshallingUtils.toJson(new SessionSnapshot("event-1", "session-1", 1, 1, response.payload()));
        assertTrue(snapshot.contains("\"snapshot\":{\"sessionId\":\"session-1\""));
        assertFalse(snapshot.contains("\"snapshot\":\""));
    }
}
