package ai.suncode.message.remote;

import com.fasterxml.jackson.annotation.JsonAlias;

import java.util.List;
import java.util.Map;

/**
 * Rust {@code SessionsResult} returned by {@code sessions.list}. {@code project_id} is snake_case on the
 * wire while {@code sessionStates} is camelCase; state values are {@code idle|running|failed|approval|question}.
 */
public record DesktopSessionsResult(
        @JsonAlias("project_id") String projectId,
        List<DesktopSessionRecord> sessions,
        Map<String, String> sessionStates,
        String encPayload) {
}
