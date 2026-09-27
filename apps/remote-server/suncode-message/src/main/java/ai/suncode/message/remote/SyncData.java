package ai.suncode.message.remote;

import java.util.List;
import java.util.Map;

public record SyncData(
        String cursor,
        boolean resetRequired,
        List<HostDto> hosts,
        List<Map<String, Object>> sessions,
        List<String> removedSessionIds,
        boolean hasMore) {
}
