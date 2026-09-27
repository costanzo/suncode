package ai.suncode.message.remote;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.databind.PropertyNamingStrategies;
import com.fasterxml.jackson.databind.annotation.JsonNaming;
import lombok.Getter;
import lombok.Setter;

import java.util.List;

/** Superset of the fields carried by all supported agent event payloads. */
@Getter
@Setter
@JsonIgnoreProperties(ignoreUnknown = true)
@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
public class EventPayload {
    private String activeTurnId;
    private List<List<String>> answers;
    private String approvalId;
    private Object arguments;
    private String callId;
    private String checkpointId;
    private String chunkBase64;
    private String code;
    private String decision;
    private Long downloadedBytes;
    private Long droppedMessages;
    private EventProviderError error;
    private String exchangeId;
    private String finishReason;
    private Long iteration;
    private Long iterations;
    private String manifestId;
    private EventMessage message;
    private String messageId;
    private String modelId;
    private String name;
    private String operation;
    private Long ordinal;
    private Long originalCharacters;
    private Long originalTokens;
    private EventMessage outputMessage;
    private String path;
    private Long position;
    private String provider;
    private String providerRequestId;
    private String providerResponseId;
    private Object questions;
    private String queuedId;
    private String queuedIdempotencyKey;
    private String reason;
    private String requestId;
    private Long restoredItems;
    private Object result;
    private Long retainedCharacters;
    private Long retainedTokens;
    private String startedAt;
    private String state;
    private String status;
    private String stream;
    private String submissionIdempotencyKey;
    private EventContextSummary summary;
    private String text;
    private List<EventTodoItem> todos;
    private String toolCallId;
    /** Count for turn.completed; call list for provider.exchange.completed. */
    private Object toolCalls;
    private String turnId;
    private Long uploadedBytes;
    private EventUsage usage;
    private String wireModel;
}
