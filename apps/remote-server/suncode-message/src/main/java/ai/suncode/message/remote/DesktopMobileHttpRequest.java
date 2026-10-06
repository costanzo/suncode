package ai.suncode.message.remote;

import com.fasterxml.jackson.annotation.JsonInclude;

import java.util.Map;

/** Values sent as the two data lines of a Desktop Mobile-request SSE event. */
@JsonInclude(JsonInclude.Include.NON_NULL)
public record DesktopMobileHttpRequest(
        Map<String, Object> pathParam,
        Map<String, Object> queryParam,
        String requestBody) {
}
