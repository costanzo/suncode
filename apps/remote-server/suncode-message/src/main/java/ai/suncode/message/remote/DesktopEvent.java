package ai.suncode.message.remote;

import com.fasterxml.jackson.annotation.JsonAlias;

import java.time.Instant;

public record DesktopEvent(
        @JsonAlias("host_id") String hostId,
        @JsonAlias("session_id") String sessionId,
        @JsonAlias("request_id") String requestId,
        @JsonAlias("event_type") String eventType,
        @JsonAlias("occurred_at") String occurredAt,
        EventPayload payload) {
}
