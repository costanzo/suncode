package ai.suncode.message.remote;

import java.util.List;

public record SyncData(
        String cursor,
        Boolean resetRequired,
        HostDto host,
        List<SessionSummaryDto> sessions,
        Boolean hasMore,
        String encPayload) {
    public static SyncData encrypted(String encPayload) {
        return new SyncData(null, null, null, null, null, encPayload);
    }
}
