package ai.suncode.message.remote;

import java.util.List;
import java.util.Map;

/** Desktop projection posted to {@code /v1/desktop/snapshot}: every project with its sessions and UI states. */
public record DesktopSnapshot(
        List<DesktopProjectRecord> projects,
        List<DesktopSessionRecord> sessions,
        Map<String, String> sessionStates,
        String encPayload) {
}
