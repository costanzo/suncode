package ai.suncode.message.remote;

import com.fasterxml.jackson.annotation.JsonInclude;

import java.util.Map;

/**
 * Envelope sent on Desktop custom Mobile-request SSE events.
 *
 * <p>Path and query values are deliberately outside the encrypted payload so
 * the Desktop can route the request after the Remote Server has only relayed
 * the opaque body. Plaintext requests use {@code requestBody}; encrypted
 * requests use {@code encPayload} and leave {@code requestBody} absent.</p>
 */
@JsonInclude(JsonInclude.Include.NON_NULL)
public record DesktopMobileHttpRequest(
        Map<String, Object> pathParam,
        Map<String, Object> queryParam,
        Object requestBody,
        String encPayload) {
}
