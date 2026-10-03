package ai.suncode.message.remote;

public record DesktopResponse(
        String requestId,
        String hostId,
        String payload) {
}
