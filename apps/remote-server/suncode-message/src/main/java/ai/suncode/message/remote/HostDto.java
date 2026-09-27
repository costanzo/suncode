package ai.suncode.message.remote;

import java.time.Instant;

public record HostDto(
        String id,
        String displayName,
        String endpoint,
        String connectionState,
        int projectCount,
        int activeSessionCount,
        Instant lastSeenAt,
        String desktopVersion,
        String protocolVersion) {
}
