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
    private static final Pattern SESSION_PATH = Pattern.compile("/v1/mobile/hosts/([^/]+)/sessions/([^/]+)(?:/.*)?$");

    private final RemoteAuthService authService;
    private final RemoteRelayService relayService;

    @Override
    public void doFilter(ServletRequest servletRequest, ServletResponse servletResponse, FilterChain chain)
            throws IOException, ServletException {
        if (!(servletRequest instanceof HttpServletRequest request) || !(servletResponse instanceof HttpServletResponse response)) {
            chain.doFilter(servletRequest, servletResponse);
            return;
        }
        String path = request.getServletPath();
        if (isPublic(request)) {
            String publicHost = request.getHeader("X-Host-Id");
            if (publicHost == null || publicHost.isBlank()) {
                FilterSupport.businessError(response, new BusinessException(PARAM_INVALID, "X-Host-Id is required"));
                return;
            }
            ServiceContext.authenticate(ClientType.MOBILE, publicHost, null);
            chain.doFilter(request, response);
            return;
        }
        final String token;
        final String hostId;
        HttpServletRequest filteredRequest = request;
        try {
            token = authService.requireMobile(request.getHeader("Authorization"));
            hostId = resolveHost(request, token, path);
            if (hostId != null) {
                authService.requireHostAccessToken(token, hostId);
            }
        } catch (BusinessException error) {
            FilterSupport.businessError(response, error);
            return;
        }
        ServiceContext.authenticate(ClientType.MOBILE, hostId, token);
        chain.doFilter(filteredRequest, response);
    }

    private boolean isPublic(HttpServletRequest request) {
        String path = request.getServletPath();
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
            return sessionMatcher.group(1);
        }
        return authService.hostForMobileToken(token);
    }
}
