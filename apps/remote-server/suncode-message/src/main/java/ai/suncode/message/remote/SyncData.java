package ai.suncode.message.remote;

import com.fasterxml.jackson.databind.JsonNode;

import java.util.List;

public record SyncData(
        String cursor,
        boolean resetRequired,
        List<HostDto> hosts,
        List<JsonNode> sessions,
        List<String> removedSessionIds,
        boolean hasMore) {
}
