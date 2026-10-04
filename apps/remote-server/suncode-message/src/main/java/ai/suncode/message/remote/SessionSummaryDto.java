package ai.suncode.message.remote;

/** Mobile {@code SessionSummary}. */
public record SessionSummaryDto(
        String id,
        String title,
        String kind,
        ProjectRefDto project,
        String state,
        String updatedAt,
        String preview,
        boolean pendingApproval,
        boolean pendingQuestion) {
}
