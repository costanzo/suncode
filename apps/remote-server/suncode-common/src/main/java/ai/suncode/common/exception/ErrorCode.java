package ai.suncode.common.exception;

import lombok.Getter;

@Getter
public enum ErrorCode {
    PARAM_INVALID(20000, "Illegal parameter"),
    INTERNAL_ERROR(30000, "Internal error"),
    ;

    private final int code;
    private final String message;

    ErrorCode(int code, String message) {
        this.code = code;
        this.message = message;
    }
}
