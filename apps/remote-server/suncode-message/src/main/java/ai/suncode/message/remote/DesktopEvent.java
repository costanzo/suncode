package ai.suncode.message.remote;

import com.fasterxml.jackson.databind.JsonNode;

import java.time.Instant;

public record DesktopEvent(
        String hostId,
        String sessionId,
        String requestId,
        String eventType,
        Instant occurredAt,
        JsonNode payload) {
}
