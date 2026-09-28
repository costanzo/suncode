package ai.suncode.common.exception;

import lombok.Getter;

@Getter
public enum ErrorCode {
    PARAM_INVALID(40000, "Illegal parameter", 400),
    UNAUTHORIZED(40100, "Unauthorized", 401),
    HOST_NOT_FOUND(40400, "host_not_found", 404),
    SESSION_NOT_FOUND(40401, "session_not_found", 404),
    PAIRING_EXPIRED(41000, "pairing_expired", 410),
    CURSOR_EXPIRED(41001, "cursor_expired", 410),
    CONFLICT(40900, "request failed", 409),
    DESKTOP_UNAVAILABLE(50300, "desktop_unavailable", 503),
    DESKTOP_REQUEST_FAILED(50400, "desktop_request_failed", 504),
    DESKTOP_TIMEOUT(50401, "desktop_timeout", 504),
    REQUEST_EXPIRED(50402, "request_expired", 504),
    INTERNAL_ERROR(50000, "Internal error", 500),
    ;

    private final int code;
    private final String message;
    private final int httpStatus;

    ErrorCode(int code, String message, int httpStatus) {
        this.code = code;
        this.message = message;
        this.httpStatus = httpStatus;
    }
}
