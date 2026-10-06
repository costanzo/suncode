package ai.suncode.message.remote;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonValue;

/** Presentation state of a paired Desktop connection. */
public enum ConnectionState {
    CONNECTED("connected"),
    CONNECTING("connecting"),
    DEGRADED("degraded"),
    OFFLINE("offline"),
    UNAUTHORIZED("unauthorized"),
    INCOMPATIBLE("incompatible");

    private final String value;

    ConnectionState(String value) {
        this.value = value;
    }

    @JsonValue
    public String value() {
        return value;
    }

    @JsonCreator
    public static ConnectionState fromValue(String value) {
        for (ConnectionState state : values()) {
            if (state.value.equals(value)) {
                return state;
            }
        }
        throw new IllegalArgumentException("Unsupported connection state: " + value);
    }
}
