package ai.suncode.message.remote;

import com.fasterxml.jackson.annotation.JsonRawValue;

public record SessionSnapshot(
        String eventId,
        String sessionId,
        long sequence,
        long sessionRevision,
        @JsonRawValue String snapshot) {
}
