package ai.suncode.message.remote;

import com.fasterxml.jackson.annotation.JsonRawValue;
import com.fasterxml.jackson.databind.PropertyNamingStrategies;
import com.fasterxml.jackson.databind.annotation.JsonNaming;

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
public record SessionSnapshot(
        String eventId,
        String sessionId,
        long sequence,
        long sessionRevision,
        @JsonRawValue String snapshot) {
}
