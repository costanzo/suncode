package ai.suncode.message.remote;

import com.fasterxml.jackson.databind.JsonNode;

public record DesktopResponse(
        String requestId,
        String hostId,
        String sessionId,
        boolean success,
        Integer code,
        String message,
        JsonNode payload) {
}
