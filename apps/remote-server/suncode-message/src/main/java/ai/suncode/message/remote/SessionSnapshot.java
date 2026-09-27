package ai.suncode.message.remote;

import com.fasterxml.jackson.databind.JsonNode;

public record SessionSnapshot(
        String eventId,
        String sessionId,
        long sequence,
        long sessionRevision,
        JsonNode snapshot) {
}
