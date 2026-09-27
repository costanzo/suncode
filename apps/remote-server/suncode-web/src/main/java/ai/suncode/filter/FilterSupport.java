package ai.suncode.filter;

import ai.suncode.common.exception.BusinessException;
import ai.suncode.common.utils.MarshallingUtils;
import ai.suncode.message.ApiBaseRet;
import jakarta.servlet.http.HttpServletResponse;
import org.springframework.http.MediaType;

import java.io.IOException;

final class FilterSupport {
    private FilterSupport() {
    }

    static void businessError(HttpServletResponse response, BusinessException error) throws IOException {
        write(response, error.getErrorCode().getHttpStatus(), error.getErrorCode().getCode(), error.getMessage());
    }

    private static void write(HttpServletResponse response, int status, int code, String message) throws IOException {
        response.setStatus(status);
        response.setCharacterEncoding("UTF-8");
        response.setContentType(MediaType.APPLICATION_JSON_VALUE);
        response.getWriter().write(MarshallingUtils.toJson(ApiBaseRet.error(code, message)));
    }
}
