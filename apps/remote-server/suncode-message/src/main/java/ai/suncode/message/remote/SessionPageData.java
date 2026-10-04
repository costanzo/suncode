package ai.suncode.message.remote;

import java.util.List;

/** Mobile session page. In end-to-end encrypted mode only {@code encPayload} is set. */
public record SessionPageData(List<SessionSummaryDto> items, String nextCursor, Boolean hasMore, String encPayload) {
    public static SessionPageData encrypted(String encPayload) {
        return new SessionPageData(null, null, null, encPayload);
    }
}
