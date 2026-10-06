package ai.suncode.message.remote;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;

/** Known event types sent across the remote-control SSE boundaries. */
public enum EventType {
    TURN_STATE("turn.state"),
    TOOL_STATE("tool.state"),
    TOOL_REQUESTED("tool.requested"),
    TOOL_RESULT("tool.result"),
    TOOL_OUTPUT("tool.output"),
    MESSAGE_USER("message.user"),
    MESSAGE_ASSISTANT("message.assistant"),
    MESSAGE_TOOL("message.tool"),
    ASSISTANT_DELTA("assistant.delta"),
    TURN_QUEUED("turn.queued"),
    TURN_COMPLETED("turn.completed"),
    USAGE_UPDATED("usage.updated"),
    CONTEXT_COMPACTED("context.compacted"),
    PROVIDER_EXCHANGE_STARTED("provider.exchange.started"),
    PROVIDER_EXCHANGE_PROGRESS("provider.exchange.progress"),
    PROVIDER_EXCHANGE_COMPLETED("provider.exchange.completed"),
    PROVIDER_EXCHANGE_FAILED("provider.exchange.failed"),
    APPROVAL_REQUESTED("approval.requested"),
    APPROVAL_RESOLVED("approval.resolved"),
    QUESTION_ASKED("question.asked"),
    QUESTION_REPLIED("question.replied"),
    QUESTION_REJECTED("question.rejected"),
    TODO_UPDATED("todo.updated"),
    CHECKPOINT_CAPTURED("checkpoint.captured"),
    CHECKPOINT_ITEM_RESTORED("checkpoint.item_restored"),
    CHECKPOINT_RESTORE_FAILED("checkpoint.restore_failed"),
    CHECKPOINT_RESTORED("checkpoint.restored"),
    MOBILE_PAIRING_EXCHANGE_SUCCEEDED("mobile.pairings.exchange.succeeded"),
    MOBILE_AUTH_LOGOUT("mobile.auth.logout");

    private final String value;

    EventType(String value) {
        this.value = value;
    }

    @JsonValue
    public String value() {
        return value;
    }

    @JsonCreator
    public static EventType fromValue(String value) {
        for (EventType eventType : values()) {
            if (eventType.value.equals(value)) {
                return eventType;
            }
        }
        throw new IllegalArgumentException("Unsupported Desktop event type: " + value);
    }
}
