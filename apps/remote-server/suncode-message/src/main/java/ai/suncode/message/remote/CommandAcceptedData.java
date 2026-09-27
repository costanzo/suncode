package ai.suncode.message.remote;

import java.time.Instant;

public record CommandAcceptedData(String requestId, Instant acceptedAt, String sessionId) {
}
