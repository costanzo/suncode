package ai.suncode.message.remote;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.databind.PropertyNamingStrategies;
import com.fasterxml.jackson.databind.annotation.JsonNaming;

@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
public record EventUsage(
        Long inputTokens,
        Long outputTokens,
        Long totalTokens,
        Long cacheReadTokens,
        Long cacheMissTokens,
        Long cacheWriteTokens,
        Long reasoningTokens) {
}
