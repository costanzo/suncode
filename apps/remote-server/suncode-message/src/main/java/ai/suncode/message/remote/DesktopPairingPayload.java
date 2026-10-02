package ai.suncode.message.remote;

public record DesktopPairingPayload(
        String hostId,
        String accessToken,
        String refreshToken,
        java.time.Instant accessTokenExpiresAt,
        String mobilePairingCode) {
}
