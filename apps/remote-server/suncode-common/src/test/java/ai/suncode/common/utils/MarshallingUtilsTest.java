package ai.suncode.common.utils;

import ai.suncode.message.remote.Command;
import ai.suncode.message.remote.CommandAcceptedData;
import ai.suncode.message.remote.ConnectionState;
import ai.suncode.message.remote.DesktopEvent;
import ai.suncode.message.remote.DesktopResponse;
import ai.suncode.message.remote.EventType;
import ai.suncode.message.remote.EventPayload;
import ai.suncode.message.remote.MobileEvent;
import ai.suncode.message.remote.SessionSnapshot;
import org.junit.jupiter.api.Test;

import java.time.Instant;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class MarshallingUtilsTest {
    @Test
    void serializesEventTimeAndConvertsRecordToTree() {
        String occurredAt = "2026-09-27T10:00:00Z";
        EventPayload payload = new EventPayload();
        payload.setTurnId("turn-1");
        payload.setState("running");
        MobileEvent event = new MobileEvent(
                "host:1", 1, "host", "session", "request", EventType.TURN_STATE, occurredAt,
                payload);

        String json = MarshallingUtils.toJson(event);
        MobileEvent restored = MarshallingUtils.fromJson(json, MobileEvent.class);
        assertEquals(occurredAt, restored.occurredAt());
        assertEquals("running", restored.payload().getState());
        assertEquals("turn-1", restored.payload().getTurnId());
        assertTrue(json.contains("\"turn_id\":\"turn-1\""));
        assertFalse(json.contains("approval_id"));
        assertEquals("2026-09-27T10:00:00Z", MarshallingUtils.convertValue(event, Map.class).get("occurred_at"));
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

        assertEquals(EventType.MESSAGE_ASSISTANT, event.eventType());
        assertTrue(MarshallingUtils.toJson(event).contains("\"eventType\":\"message.assistant\""));
        assertEquals("turn-1", event.payload().getTurnId());
        assertEquals("Hello", event.payload().getMessage().content().get(0).text());
    }

    @Test
    void serializesAndReadsProtocolEnumsAsTheirWireValues() {
        assertEquals("\"session.get\"", MarshallingUtils.toJson(Command.SESSION_GET));
        assertEquals(Command.SESSION_GET, MarshallingUtils.fromJson("\"session.get\"", Command.class));
        assertEquals("\"connected\"", MarshallingUtils.toJson(ConnectionState.CONNECTED));
        assertEquals(ConnectionState.CONNECTED,
                MarshallingUtils.fromJson("\"connected\"", ConnectionState.class));
        assertEquals(EventType.MESSAGE_ASSISTANT,
                MarshallingUtils.fromJson("\"message.assistant\"", EventType.class));
    }

    @Test
    void convertsCommandResponseData() {
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
