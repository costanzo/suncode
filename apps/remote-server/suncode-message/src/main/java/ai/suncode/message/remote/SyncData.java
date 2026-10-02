package ai.suncode.message.remote;

public record SyncData(
        String cursor,
        boolean resetRequired,
        HostDto host,
        java.util.List<java.util.Map<String, Object>> sessions,
        boolean hasMore) {
}
