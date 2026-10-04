package ai.suncode.filter;

import ai.suncode.message.remote.CachedBodyRequest;
import jakarta.servlet.*;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.AntPathMatcher;
import org.springframework.util.PathMatcher;
import org.springframework.web.util.ContentCachingResponseWrapper;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.Map;

@Slf4j
public class RequestResponseLoggingFilter implements Filter {
    private static final PathMatcher PATH_MATCHER = new AntPathMatcher();
    private static final int MAX_PAYLOAD_LENGTH = 2048;

    // Skip some urls if needed
    private boolean shouldSkipLogging(String uri) {
        // Example: skip logging for health check endpoint
        return uri.startsWith("/actuator");
    }

    private boolean shouldSkipCachingSSE(String uri) {
        // Example: skip caching for large file downloads
        return uri.equals("/v1/desktop/events") ||
                PATH_MATCHER.match("/v1/mobile/hosts/{hostId}/sessions/{sessionId}/events", uri);
    }

    @Override
    public void doFilter(ServletRequest request, ServletResponse response, FilterChain chain)
            throws IOException, ServletException {

        long startTime = System.currentTimeMillis();
        CachedBodyRequest requestWrapper = new CachedBodyRequest((HttpServletRequest) request);

        if (shouldSkipCachingSSE(requestWrapper.getServletPath())) {
            chain.doFilter(requestWrapper, response);
            return;
        }

        // Read and retain the body before entering the chain. Unlike
        // ContentCachingRequestWrapper, this makes the payload available for
        // the entry log while still allowing Spring MVC to read it later.
        if (!shouldSkipLogging(requestWrapper.getServletPath())) {
            try {
                logRequest(requestWrapper, requestWrapper.body());
            } catch (Exception e) {
                log.error("Error logging request", e);
            }
        }

        ContentCachingResponseWrapper responseWrapper = new ContentCachingResponseWrapper((HttpServletResponse) response);
        try {
            chain.doFilter(requestWrapper, responseWrapper);
        } finally {
            long endTime = System.currentTimeMillis();
            long duration = endTime - startTime;
            if (!shouldSkipLogging(requestWrapper.getServletPath())) {
                try {
                    logResponse(responseWrapper, duration);
                } catch (Exception e) {
                    log.error("Error logging response", e);
                }
            }
            responseWrapper.copyBodyToResponse();
        }
    }

    private void logRequest(HttpServletRequest request, byte[] requestBody) {
        String queryString = request.getQueryString() != null ? request.getQueryString() : "";

        String contentType = request.getContentType();
        String requestBodyStr;
        if (isMultipartRequest(contentType)) {
            requestBodyStr = String.format("[Multipart request, Content-Length: %d]", request.getContentLength());
        } else {
            if (requestBody.length == 0 &&
                    contentType != null &&
                    contentType.toLowerCase().contains("application/x-www-form-urlencoded")) {

                requestBodyStr = buildFormDataString(request);
            } else {
                requestBodyStr = new String(requestBody, StandardCharsets.UTF_8);

                if (requestBodyStr.trim().isEmpty() && contentType != null &&
                        (contentType.toLowerCase().contains("json") ||
                                contentType.toLowerCase().contains("xml"))) {
                    requestBodyStr = "[Empty " + contentType.split(";")[0] + "]";
                }

                if (requestBodyStr.length() > MAX_PAYLOAD_LENGTH) {
                    requestBodyStr = requestBodyStr.substring(0, MAX_PAYLOAD_LENGTH) + "...";
                }
            }
        }

        log.info("REQUEST ==> Method: {}, URL: {}, QueryParams: {}, Body: {}, IP: {}",
                request.getMethod(), request.getServletPath(), queryString, requestBodyStr, getIpAddress(request));
    }

    private boolean isMultipartRequest(String contentType) {
        return contentType != null && contentType.toLowerCase().startsWith("multipart/");
    }

    private void logResponse(ContentCachingResponseWrapper response, long duration) {
        // 获取响应体
        byte[] responseBody = response.getContentAsByteArray();
        String responseBodyStr = buildResponseBodyLogMessage(responseBody, response.getContentType());

        log.info("RESPONSE <== Status: {}, Body: {}, cost: {}ms",
                response.getStatus(), responseBodyStr, duration);
    }

    private String buildResponseBodyLogMessage(byte[] responseBody, String contentType) {
        if (isBinaryContentType(contentType)) {
            return String.format("[Binary body omitted, contentType=%s, size=%d bytes]",
                    contentType, responseBody.length);
        }

        String responseBodyStr = new String(responseBody, StandardCharsets.UTF_8);
        if (responseBodyStr.length() > MAX_PAYLOAD_LENGTH) {
            responseBodyStr = responseBodyStr.substring(0, MAX_PAYLOAD_LENGTH) + "...";
        }
        return responseBodyStr;
    }

    private boolean isBinaryContentType(String contentType) {
        if (contentType == null) {
            return false;
        }

        String normalizedContentType = contentType.toLowerCase();
        return normalizedContentType.startsWith("image/")
                || normalizedContentType.startsWith("audio/")
                || normalizedContentType.startsWith("video/")
                || normalizedContentType.contains("octet-stream")
                || normalizedContentType.contains("pdf")
                || normalizedContentType.contains("zip")
                || normalizedContentType.contains("excel")
                || normalizedContentType.contains("spreadsheet")
                || normalizedContentType.contains("msword")
                || normalizedContentType.contains("wordprocessingml")
                || normalizedContentType.contains("powerpoint")
                || normalizedContentType.contains("presentationml");
    }

    private String buildFormDataString(HttpServletRequest request) {
        Map<String, String[]> parameterMap = request.getParameterMap();

        if (parameterMap.isEmpty()) {
            return "[No form data]";
        }

        StringBuilder sb = new StringBuilder();
        for (Map.Entry<String, String[]> entry : parameterMap.entrySet()) {
            String key = entry.getKey();
            String[] values = entry.getValue();

            if (key.contains("_csrf") || key.contains("remember-me")) {
                continue;
            }

            if (sb.length() > 0) {
                sb.append("&");
            }

            sb.append(key).append("=");
            if (values.length == 1) {
                if (key.toLowerCase().contains("password") || key.toLowerCase().contains("pwd")) {
                    sb.append("***");
                } else {
                    sb.append(truncateIfNeeded(values[0]));
                }
            } else {
                sb.append(Arrays.toString(values));
            }
        }

        String result = sb.toString();
        if (result.length() > MAX_PAYLOAD_LENGTH) {
            result = result.substring(0, MAX_PAYLOAD_LENGTH) + "...";
        }
        return result;
    }

    private String truncateIfNeeded(String value) {
        if (value.length() > 50) {
            return value.substring(0, 50) + "...";
        }
        return value;
    }

    public static String getIpAddress(HttpServletRequest request) {
        String ip = request.getHeader("x-forwarded-for");
        if (ip == null || ip.length() == 0 || "unknown".equalsIgnoreCase(ip)) {
            ip = request.getHeader("Proxy-Client-IP");
        }
        if (ip == null || ip.length() == 0 || "unknown".equalsIgnoreCase(ip)) {
            ip = request.getHeader("WL-Proxy-Client-IP");
        }
        if (ip == null || ip.length() == 0 || "unknown".equalsIgnoreCase(ip)) {
            ip = request.getHeader("HTTP_CLIENT_IP");
        }
        if (ip == null || ip.length() == 0 || "unknown".equalsIgnoreCase(ip)) {
            ip = request.getHeader("HTTP_X_FORWARDED_FOR");
        }
        if (ip == null || ip.length() == 0 || "unknown".equalsIgnoreCase(ip)) {
            ip = request.getRemoteAddr();
        }
        return extractRealIp(ip);
    }

    private static String extractRealIp(String ip) {
        if (ip != null && ip.contains(",")) {
            String[] ips = ip.split(",");
            for (String tmpIp : ips) {
                tmpIp = tmpIp.trim();
                if (!isLocalhostIp(tmpIp)) {
                    return tmpIp;
                }
            }
            return ips[0].trim();
        }
        return ip;
    }

    private static boolean isLocalhostIp(String ip) {
        return "127.0.0.1".equals(ip) ||
                "0:0:0:0:0:0:0:1".equals(ip) ||
                "localhost".equalsIgnoreCase(ip) ||
                ip.startsWith("192.168.") ||
                ip.startsWith("10.") ||
                ip.startsWith("172.16.") ||
                ip.startsWith("172.17.") ||
                ip.startsWith("172.18.") ||
                ip.startsWith("172.19.") ||
                ip.startsWith("172.20.") ||
                ip.startsWith("172.21.") ||
                ip.startsWith("172.22.") ||
                ip.startsWith("172.23.") ||
                ip.startsWith("172.24.") ||
                ip.startsWith("172.25.") ||
                ip.startsWith("172.26.") ||
                ip.startsWith("172.27.") ||
                ip.startsWith("172.28.") ||
                ip.startsWith("172.29.") ||
                ip.startsWith("172.30.") ||
                ip.startsWith("172.31.");
    }
}
