package ai.suncode.message.remote;

import com.fasterxml.jackson.databind.PropertyNamingStrategies;
import com.fasterxml.jackson.databind.annotation.JsonNaming;

import java.time.Instant;

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
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
