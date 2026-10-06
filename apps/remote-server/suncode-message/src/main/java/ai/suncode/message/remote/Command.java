package ai.suncode.message.remote;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;

/** Operations accepted by the Desktop command boundary. */
public enum Command {
    PROJECTS_LIST("projects.list"),
    SESSIONS_LIST("sessions.list"),
    SESSION_CREATE("session.create"),
    SESSION_GET("session.get"),
    SESSION_MESSAGE("session.message"),
    SESSION_SEND_MESSAGE("session.send_message"),
    APPROVAL_RESOLVE("approval.resolve"),
    QUESTION_REPLY("question.reply"),
    TURN_CANCEL("turn.cancel"),
    SESSION_CANCEL("session.cancel"),
    TURN_RETRY("turn.retry"),
    SESSION_RETRY("session.retry");

    private final String value;

    Command(String value) {
        this.value = value;
    }

    @JsonValue
    public String value() {
        return value;
    }

    @JsonCreator
    public static Command fromValue(String value) {
        for (Command command : values()) {
            if (command.value.equals(value)) {
                return command;
            }
        }
        throw new IllegalArgumentException("Unsupported Desktop command: " + value);
    }

    public boolean isApproval() {
        return this == APPROVAL_RESOLVE;
    }

    public boolean isQuestion() {
        return this == QUESTION_REPLY;
    }
}
