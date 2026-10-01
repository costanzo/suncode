package ai.suncode.filter;

import ai.suncode.common.exception.BusinessException;
import ai.suncode.common.http.ServiceContext;
import ai.suncode.common.utils.MarshallingUtils;
import ai.suncode.message.remote.CachedBodyRequest;
import ai.suncode.message.remote.ClientType;
import ai.suncode.message.remote.CreateSessionRequest;
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

import java.io.IOException;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import static ai.suncode.common.exception.ErrorCode.PARAM_INVALID;

@RequiredArgsConstructor
public class MobileAuthFilter implements Filter {
    private static final Pattern HOST_PATH = Pattern.compile("/v1/mobile/hosts/([^/]+)(?:/.*)?$");
    private static final Pattern SESSION_PATH = Pattern.compile("/v1/mobile/sessions/([^/]+)(?:/.*)?$");

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
        if (isPublic(request)) {
            chain.doFilter(request, response);
            return;
        }
        final String token;
        final String hostId;
        HttpServletRequest filteredRequest = request;
        try {
            token = authService.requireMobile(request.getHeader("Authorization"));
            if ("POST".equalsIgnoreCase(request.getMethod()) && "/v1/mobile/sessions".equals(path)) {
                CachedBodyRequest wrapped = new CachedBodyRequest(request);
                CreateSessionRequest body;
                try {
                    body = MarshallingUtils.fromJson(wrapped.body(), CreateSessionRequest.class);
                } catch (BusinessException error) {
                    throw new BusinessException(PARAM_INVALID, "invalid request body", error);
                }
                if (body == null || body.hostId() == null || body.hostId().isBlank()) {
                    throw new BusinessException(PARAM_INVALID, "hostId is required");
                }
                filteredRequest = wrapped;
                hostId = body.hostId();
            } else {
                hostId = resolveHost(request, token, path);
            }
            if (hostId != null) {
                authService.requireHostAccessToken(token, hostId);
            }
        } catch (BusinessException error) {
            FilterSupport.businessError(response, error);
            return;
        } catch (IOException error) {
            FilterSupport.businessError(response, new BusinessException(PARAM_INVALID, "invalid request body"));
            return;
        }
        ServiceContext.authenticate(ClientType.MOBILE, hostId, token);
        chain.doFilter(filteredRequest, response);
    }

    private boolean isPublic(HttpServletRequest request) {
        String path = applicationPath(request);
        return "/v1/mobile/health".equals(path)
                || "/v1/mobile/pairings/exchange".equals(path)
                || "/v1/mobile/auth/refresh".equals(path);
    }

    private String resolveHost(HttpServletRequest request, String token, String path) {
        Matcher hostMatcher = HOST_PATH.matcher(path);
        if (hostMatcher.matches()) {
            return hostMatcher.group(1);
        }
        Matcher sessionMatcher = SESSION_PATH.matcher(path);
        if (sessionMatcher.matches()) {
            return relayService.hostForSession(sessionMatcher.group(1));
        }
        if ("GET".equalsIgnoreCase(request.getMethod()) && "/v1/mobile/sessions".equals(path)) {
            String selectedHost = request.getParameter("hostId");
            if (selectedHost != null && !selectedHost.isBlank()) {
                return selectedHost;
            }
        }
        return authService.hostForMobileToken(token);
    }

    private static String applicationPath(HttpServletRequest request) {
        String uri = request.getRequestURI();
        String contextPath = request.getContextPath();
        return contextPath != null && !contextPath.isEmpty() && uri.startsWith(contextPath)
                ? uri.substring(contextPath.length()) : uri;
    }

}
