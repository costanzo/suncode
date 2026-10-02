package ai.suncode.message.remote;

import java.util.Map;

/** Generic plaintext envelope sent on the Desktop custom Mobile-request SSE events. */
public record DesktopMobileHttpRequest(
        Map<String, Object> pathParam,
        Map<String, Object> queryParam,
        Object requestBody) {
}
