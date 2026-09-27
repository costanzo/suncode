package ai.suncode.common.exception;

import lombok.Getter;

@Getter
public enum ErrorCode {
    PARAM_INVALID(20000, "Illegal parameter", 400),
    UNAUTHORIZED(40100, "Unauthorized", 401),
    HOST_NOT_FOUND(404, "host_not_found", 404),
    SESSION_NOT_FOUND(404, "session_not_found", 404),
    PAIRING_EXPIRED(410, "pairing_expired", 410),
    CURSOR_EXPIRED(410, "cursor_expired", 410),
    CONFLICT(409, "request failed", 409),
    DESKTOP_UNAVAILABLE(503, "desktop_unavailable", 503),
    DESKTOP_REQUEST_FAILED(504, "desktop_request_failed", 504),
    DESKTOP_TIMEOUT(504, "desktop_timeout", 504),
    REQUEST_EXPIRED(504, "request_expired", 504),
    INTERNAL_ERROR(30000, "Internal error", 500),
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
