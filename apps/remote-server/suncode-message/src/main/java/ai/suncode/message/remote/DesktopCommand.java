package ai.suncode.message.remote;

public record DesktopCommand(
        String requestId,
        String hostId,
        String sessionId,
        String command,
        DesktopCommandPayload payload) {
}
