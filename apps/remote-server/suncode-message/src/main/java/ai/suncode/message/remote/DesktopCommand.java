package ai.suncode.message.remote;

import com.fasterxml.jackson.databind.JsonNode;

public record DesktopCommand(
        String requestId,
        String hostId,
        String sessionId,
        String command,
        JsonNode payload) {
}
