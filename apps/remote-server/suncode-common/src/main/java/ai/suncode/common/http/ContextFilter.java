package ai.suncode.common.http;

import jakarta.servlet.Filter;
import jakarta.servlet.FilterChain;
import jakarta.servlet.ServletException;
import jakarta.servlet.ServletRequest;
import jakarta.servlet.ServletResponse;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import org.slf4j.MDC;
import org.springframework.util.StringUtils;

import java.io.IOException;

import static ai.suncode.common.utils.TracingUtils.createNewLogId;
import static ai.suncode.common.utils.TracingUtils.createNewRequestId;


public class ContextFilter implements Filter {
    public static final String MDC_KEY_LOG_ID = "logId";
    public static final String MDC_KEY_REQUEST_ID = "requestId";
    public static final String LOG_ID_HEADER = "x-log-id";
    public static final String REQUEST_ID_HEADER = "x-request-id";

    @Override
    public void doFilter(ServletRequest servletRequest, ServletResponse servletResponse, FilterChain filterChain) throws IOException, ServletException {
        if (!(servletRequest instanceof HttpServletRequest request) || !(servletResponse instanceof HttpServletResponse response)) {
            filterChain.doFilter(servletRequest, servletResponse);
            return;
        }

        String logId = request.getHeader(LOG_ID_HEADER);
        if (!StringUtils.hasText(logId)) {
            logId = createNewLogId();
        }
        String requestId = request.getHeader(REQUEST_ID_HEADER);
        if (!StringUtils.hasText(requestId)) {
            requestId = createNewRequestId();
        }

        MDC.put(MDC_KEY_LOG_ID, logId);
        MDC.put(MDC_KEY_REQUEST_ID, requestId);
        response.addHeader(LOG_ID_HEADER, logId);

        try {
            filterChain.doFilter(servletRequest, servletResponse);
        } finally {
            MDC.clear();
        }
    }
}
