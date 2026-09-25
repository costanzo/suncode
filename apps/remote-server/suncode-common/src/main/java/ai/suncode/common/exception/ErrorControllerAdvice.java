package ai.suncode.common.exception;

import ai.suncode.message.ApiBaseRet;
import lombok.extern.slf4j.Slf4j;
import org.springframework.http.HttpStatus;
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
}
