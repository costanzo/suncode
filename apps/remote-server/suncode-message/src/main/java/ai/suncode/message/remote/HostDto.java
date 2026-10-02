package ai.suncode.message.remote;

import java.time.Instant;

public record HostDto(
        String id,
        String displayName,
        String connectionState,
        Instant lastSeenAt,
        String agentVersion) {
}
