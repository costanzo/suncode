package ai.suncode.message.remote;

import java.time.Instant;

public record PairingExchangeData(
        String accessToken,
        String refreshToken,
        Instant accessTokenExpiresAt,
        HostDto host) {
}
