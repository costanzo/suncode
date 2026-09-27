package ai.suncode.message.remote;

import java.time.Instant;

public record MobileEvent(
        String eventId,
        long sequence,
        String hostId,
        String sessionId,
        String requestId,
        String eventType,
        Instant occurredAt,
        EventPayload payload) {
}
