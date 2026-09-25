package ai.suncode.common.exception;

import lombok.Getter;

@Getter
public enum ErrorCode {
    PARAM_INVALID(20000, "参数不合法"),
    INTERNAL_ERROR(30000, "内部错误"),
    ;

    private final int code;
    private final String message;

    ErrorCode(int code, String message) {
        this.code = code;
        this.message = message;
    }
}
