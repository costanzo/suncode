package ai.suncode.message.enums;

import com.fasterxml.jackson.annotation.JsonValue;
import lombok.Getter;

@Getter
public enum ChatMessageTypeEnum {
    TEXT("text"),
    AUDIO("audio"),
    IMAGE("image"),
    ERROR("error"),
    ;
    private final String type;

    ChatMessageTypeEnum(String type) {
        this.type = type;
    }

    @JsonValue
    public String getType() {
        return type;
    }
}
