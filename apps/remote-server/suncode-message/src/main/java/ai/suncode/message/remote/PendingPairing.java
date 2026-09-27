package ai.suncode.message.remote;

public record PendingPairing(String hostId, String displayName, long expiresAtNanos) {
}
