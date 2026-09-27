package ai.suncode.message.remote;

import com.fasterxml.jackson.databind.JsonNode;

import java.util.List;

public record SessionPageData(List<JsonNode> items, String nextCursor, boolean hasMore) {
}
