package ai.suncode.common.exception;

import lombok.Getter;

@Getter
public abstract class BaseException extends RuntimeException{
    private final ErrorCode errorCode;
    private final String message;

    public BaseException(ErrorCode errorCode) {
        super(errorCode.getMessage());
        this.errorCode = errorCode;
        this.message = errorCode.getMessage();
    }

    public BaseException(ErrorCode errorCode, String message) {
        super(message);
        this.errorCode = errorCode;
        this.message = message;
    }

}
