package ai.suncode.message.remote;

/** Rust {@code ProjectRecord}. {@code canonicalRoot} is absolute and must never be forwarded to Mobile. */
public record DesktopProjectRecord(
        String projectId,
        String displayName,
        String canonicalRoot,
        String createdAt,
        String updatedAt,
        String lastOpenedAt,
        String archivedAt) {
}
