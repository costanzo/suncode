package ai.suncode.message.remote;

public record DesktopPairingPayload(
        String hostId,
        String desktopToken,
        String mobilePairingPayload,
        String eventsUrl,
        String requestsUrl,
        String resultsUrl) {
}
