package ai.suncode.message.remote;

import java.util.List;
import java.util.Map;

public record SessionPageData(List<Map<String, Object>> items, String nextCursor, boolean hasMore) {
}
