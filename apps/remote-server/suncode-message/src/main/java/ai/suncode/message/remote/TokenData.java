package ai.suncode.message.remote;

import java.time.Instant;

public record TokenData(String accessToken, String refreshToken, Instant accessTokenExpiresAt) {
}
