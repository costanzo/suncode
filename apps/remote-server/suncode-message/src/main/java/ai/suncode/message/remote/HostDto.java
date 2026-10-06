package ai.suncode.message.remote;

import java.time.Instant;

public record HostDto(
        String id,
        String displayName,
        ConnectionState connectionState,
        Instant lastSeenAt,
        String agentVersion) {
}
