package ai.suncode.common.exception;

import ai.suncode.message.ApiBaseRet;
import lombok.extern.slf4j.Slf4j;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.MethodArgumentNotValidException;
import org.springframework.web.bind.annotation.ControllerAdvice;
import org.springframework.web.bind.annotation.ExceptionHandler;
import org.springframework.web.bind.annotation.ResponseBody;
import org.springframework.web.bind.annotation.ResponseStatus;

import static ai.suncode.common.exception.ErrorCode.INTERNAL_ERROR;
import static ai.suncode.common.exception.ErrorCode.PARAM_INVALID;

@ControllerAdvice
@Slf4j
public class ErrorControllerAdvice {

    @ExceptionHandler(value = MethodArgumentNotValidException.class)
    @ResponseStatus(HttpStatus.BAD_REQUEST)
    @ResponseBody
    public ApiBaseRet<?> methodNotValidException(MethodArgumentNotValidException e) {
        log.info("MethodArgumentNotValidException error ", e);
        return ApiBaseRet.error(PARAM_INVALID.getCode(), PARAM_INVALID.getMessage());
    }

    @ExceptionHandler(value = BusinessException.class)
    @ResponseStatus(HttpStatus.BAD_REQUEST)
    @ResponseBody
    public ApiBaseRet<?> businessException(BusinessException e) {
        log.info("BusinessException error ", e);
        return ApiBaseRet.error(e.getErrorCode().getCode(), e.getMessage());
    }

    @ExceptionHandler(value = Exception.class)
    @ResponseStatus(HttpStatus.INTERNAL_SERVER_ERROR)
    @ResponseBody
    public ApiBaseRet<?> exception(Exception e) {
        log.error("Unknown error ", e);
        return ApiBaseRet.error(INTERNAL_ERROR.getCode(), INTERNAL_ERROR.getMessage());
    }

    @ExceptionHandler(value = SecurityException.class)
    @ResponseStatus(HttpStatus.UNAUTHORIZED)
    @ResponseBody
    public ApiBaseRet<?> securityException(SecurityException e) {
        return ApiBaseRet.error(40100, e.getMessage());
    }

    @ExceptionHandler(value = IllegalArgumentException.class)
    @ResponseStatus(HttpStatus.BAD_REQUEST)
    @ResponseBody
    public ApiBaseRet<?> illegalArgumentException(IllegalArgumentException e) {
        return ApiBaseRet.error(PARAM_INVALID.getCode(), e.getMessage());
    }

    @ExceptionHandler(value = IllegalStateException.class)
    @ResponseBody
    public ResponseEntity<ApiBaseRet<?>> illegalStateException(IllegalStateException e) {
        String message = e.getMessage() == null ? "request failed" : e.getMessage();
        HttpStatus status = switch (message) {
            case "desktop_unavailable" -> HttpStatus.SERVICE_UNAVAILABLE;
            case "desktop_request_failed", "desktop_timeout", "request_expired" -> HttpStatus.GATEWAY_TIMEOUT;
            case "cursor_expired" -> HttpStatus.GONE;
            case "host_not_found", "session_not_found" -> HttpStatus.NOT_FOUND;
            default -> HttpStatus.CONFLICT;
        };
        return ResponseEntity.status(status).body(ApiBaseRet.error(status.value(), message));
    }
}
