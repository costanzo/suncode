package ai.suncode.common.http;

import ai.suncode.message.remote.ClientType;
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

import static ai.suncode.common.utils.TracingUtils.createNewRequestId;


public class ContextFilter implements Filter {
    public static final String MDC_KEY_REQUEST_ID = "requestId";
    public static final String MDC_KEY_HOST_ID = "hostId";
    public static final String REQUEST_ID_HEADER = "x-request-id";
    public static final String HOST_ID_HEADER = "x-host-id";

    @Override
    public void doFilter(ServletRequest servletRequest, ServletResponse servletResponse, FilterChain filterChain) throws IOException, ServletException {
        if (!(servletRequest instanceof HttpServletRequest request) || !(servletResponse instanceof HttpServletResponse response)) {
            filterChain.doFilter(servletRequest, servletResponse);
            return;
        }

        String hostId = request.getHeader(HOST_ID_HEADER);
        String requestId = request.getHeader(REQUEST_ID_HEADER);
        if (!StringUtils.hasText(requestId)) {
            requestId = createNewRequestId();
        }

        MDC.put(MDC_KEY_HOST_ID, hostId);
        MDC.put(MDC_KEY_REQUEST_ID, requestId);
        ServiceContext.setCurrent(new ServiceContext(clientType(request), hostId, null, requestId));

        try {
            filterChain.doFilter(servletRequest, servletResponse);
        } finally {
            ServiceContext.clear();
            MDC.clear();
        }
    }

    private static ClientType clientType(HttpServletRequest request) {
        String path = applicationPath(request);
        if (path.startsWith("/v1/desktop/")) {
            return ClientType.DESKTOP;
        }
        if (path.startsWith("/v1/mobile/")) {
            return ClientType.MOBILE;
        }
        return ClientType.OTHER;
    }

    private static String applicationPath(HttpServletRequest request) {
        String uri = request.getRequestURI();
        String contextPath = request.getContextPath();
        return contextPath != null && !contextPath.isEmpty() && uri.startsWith(contextPath)
                ? uri.substring(contextPath.length()) : uri;
    }
}
