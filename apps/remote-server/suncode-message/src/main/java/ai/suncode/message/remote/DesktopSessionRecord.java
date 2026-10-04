package ai.suncode.message.remote;

/** Rust {@code SessionRecord} (camelCase). */
public record DesktopSessionRecord(
        String sessionId,
        String projectId,
        String title,
        String modelId,
        String reasoningEffort,
        String kind,
        String parentSessionId,
        String status,
        String createdAt,
        String updatedAt,
        String lastActivityAt,
        String archivedAt,
        String pinAt,
        String encPayload) {
}
