package ai.suncode.message.remote;

import com.fasterxml.jackson.annotation.JsonRawValue;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;

public record DesktopResponse(
        String requestId,
        String hostId,
        String sessionId,
        boolean success,
        Integer code,
        String message,
        @JsonRawValue @JsonDeserialize(using = RawJsonObjectDeserializer.class) String payload) {
}
