package ai.suncode.filter;

import ai.suncode.common.exception.BusinessException;
import ai.suncode.common.http.ServiceContext;
import ai.suncode.message.remote.ClientType;
import ai.suncode.service.RemoteAuthService;
import ai.suncode.service.RemoteRelayService;
import jakarta.servlet.Filter;
import jakarta.servlet.FilterChain;
import jakarta.servlet.ServletException;
import jakarta.servlet.ServletRequest;
import jakarta.servlet.ServletResponse;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import lombok.RequiredArgsConstructor;
import org.springframework.util.StringUtils;

import java.io.IOException;

import static ai.suncode.common.exception.ErrorCode.PARAM_INVALID;

@RequiredArgsConstructor
public class DesktopAuthFilter implements Filter {
    private final RemoteAuthService authService;
    private final RemoteRelayService relayService;

    @Override
    public void doFilter(ServletRequest servletRequest, ServletResponse servletResponse, FilterChain chain)
            throws IOException, ServletException {
        if (!(servletRequest instanceof HttpServletRequest request) || !(servletResponse instanceof HttpServletResponse response)) {
            chain.doFilter(servletRequest, servletResponse);
            return;
        }
        String path = applicationPath(request);
        String hostId = request.getHeader("X-Host-Id");
        String token;
        try {
            if ("POST".equalsIgnoreCase(request.getMethod())
                    && path.endsWith("/pairings")) {
                chain.doFilter(request, response);
                return;
            }
            if ("POST".equalsIgnoreCase(request.getMethod()) && path.endsWith("/auth/refresh")) {
                if (!StringUtils.hasText(hostId)) {
                    throw new BusinessException(PARAM_INVALID, "X-Host-Id is required");
                }
                ServiceContext.authenticate(ClientType.DESKTOP, hostId, null);
                chain.doFilter(request, response);
                return;
            }
            if (!StringUtils.hasText(hostId)) {
                throw new BusinessException(PARAM_INVALID, "X-Host-Id is required");
            }
            token = authService.requireDesktop(request.getHeader("Authorization"), hostId);
            if (!("GET".equalsIgnoreCase(request.getMethod()) && path.endsWith("/events"))) {
                relayService.requireDesktopConnection(hostId, token);
            }
        } catch (BusinessException error) {
            FilterSupport.businessError(response, error);
            return;
        }
        ServiceContext.authenticate(ClientType.DESKTOP, hostId, token);
        chain.doFilter(request, response);
    }

    private static String applicationPath(HttpServletRequest request) {
        String uri = request.getRequestURI();
        String contextPath = request.getContextPath();
        return contextPath != null && !contextPath.isEmpty() && uri.startsWith(contextPath)
                ? uri.substring(contextPath.length()) : uri;
    }
}
