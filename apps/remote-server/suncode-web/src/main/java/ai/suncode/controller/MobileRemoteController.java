package ai.suncode.controller;

import ai.suncode.message.ApiBaseRet;
import ai.suncode.message.remote.RemoteProtocol;
import ai.suncode.service.RemoteAuthService;
import ai.suncode.service.RemoteRelayService;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.JsonNodeFactory;
import lombok.RequiredArgsConstructor;
import org.springframework.http.HttpStatus;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.ExceptionHandler;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestHeader;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

import java.time.Instant;
import java.util.List;
import java.util.UUID;
import java.util.concurrent.TimeoutException;

@RestController
@RequestMapping("/v1")
@RequiredArgsConstructor
public class MobileRemoteController {
    private final RemoteAuthService authService;
    private final RemoteRelayService relayService;
    private final ObjectMapper objectMapper;

    @GetMapping("/health")
    public ApiBaseRet<?> health() {
        return ApiBaseRet.success(new Health("ok", Instant.now()));
    }

    @PostMapping("/pairings/exchange")
    public ApiBaseRet<?> exchange(@RequestBody RemoteProtocol.PairingExchangeRequest request) {
        return ApiBaseRet.success(authService.exchange(request));
    }

    @PostMapping("/auth/refresh")
    public ApiBaseRet<?> refresh(@RequestBody RemoteProtocol.RefreshTokenRequest request) {
        return ApiBaseRet.success(authService.refresh(request));
    }

    @PostMapping("/auth/logout")
    public ResponseEntity<Void> logout(@RequestHeader("Authorization") String authorization) {
        authService.requireMobile(authorization);
        authService.logout(authorization);
        return ResponseEntity.noContent().build();
    }

    @GetMapping("/hosts/{hostId}")
    public ApiBaseRet<?> host(@RequestHeader("Authorization") String authorization, @PathVariable String hostId) {
        authService.requireHostAccess(authorization, hostId);
        return ApiBaseRet.success(relayService.hosts().stream().filter(host -> host.id().equals(hostId)).findFirst()
                .orElseThrow(() -> new IllegalStateException("host_not_found")));
    }

    @GetMapping("/hosts/{hostId}/projects")
    public ApiBaseRet<?> projects(@RequestHeader("Authorization") String authorization, @PathVariable String hostId) throws Exception {
        authService.requireHostAccess(authorization, hostId);
        RemoteProtocol.DesktopResponse response = relayService.request(hostId, null, "projects.list", empty(), null);
        return responseEnvelope(response, RemoteProtocol.ProjectsData.class);
    }

    @GetMapping("/sessions")
    public ApiBaseRet<?> sessions(@RequestHeader("Authorization") String authorization,
                                  @RequestParam(required = false) String hostId,
                                  @RequestParam(required = false) String projectId,
                                  @RequestParam(required = false) String cursor,
                                  @RequestParam(required = false) Integer limit) throws Exception {
        authService.requireMobile(authorization);
        String selectedHost = hostId;
        if (selectedHost == null || selectedHost.isBlank()) {
            List<RemoteProtocol.HostDto> pairedHosts = relayService.hosts().stream()
                    .filter(host -> authService.isPairedWith(authorization, host.id())).toList();
            if (pairedHosts.isEmpty()) return ApiBaseRet.success(new RemoteProtocol.SessionPageData(List.of(), null, false));
            selectedHost = pairedHosts.get(0).id();
        }
        authService.requireHostAccess(authorization, selectedHost);
        JsonNode payload = objectMapper.createObjectNode()
                .put("projectId", projectId == null ? "" : projectId)
                .put("cursor", cursor == null ? "" : cursor)
                .put("limit", limit == null ? 50 : limit);
        return responseEnvelope(relayService.request(selectedHost, null, "sessions.list", payload, null), RemoteProtocol.SessionPageData.class);
    }

    @PostMapping("/sessions")
    public ResponseEntity<ApiBaseRet<?>> createSession(@RequestHeader("Authorization") String authorization,
                                                       @RequestHeader("Idempotency-Key") String idempotencyKey,
                                                       @RequestBody RemoteProtocol.CreateSessionRequest request) throws Exception {
        authService.requireHostAccess(authorization, request.hostId());
        RemoteProtocol.DesktopResponse response = relayService.request(request.hostId(), null, "session.create", objectMapper.valueToTree(request), idempotencyKey);
        String sessionId = text(response.payload(), "sessionId");
        relayService.rememberSession(sessionId, request.hostId());
        return ResponseEntity.status(HttpStatus.CREATED).body(ApiBaseRet.success(new RemoteProtocol.CommandAcceptedData(
                response.requestId(), Instant.now(), sessionId)));
    }

    @GetMapping("/sessions/{sessionId}")
    public ApiBaseRet<?> session(@RequestHeader("Authorization") String authorization, @PathVariable String sessionId) throws Exception {
        String hostId = requireHost(sessionId);
        authService.requireHostAccess(authorization, hostId);
        return responseEnvelope(relayService.request(hostId, sessionId, "session.get", empty(), null), JsonNode.class);
    }

    @GetMapping(value = "/sessions/{sessionId}/events", produces = MediaType.TEXT_EVENT_STREAM_VALUE)
    public SseEmitter sessionEvents(@RequestHeader("Authorization") String authorization,
                                    @PathVariable String sessionId,
                                    @RequestHeader(value = "Last-Event-ID", required = false) String lastEventId) throws Exception {
        authService.requireMobile(authorization);
        authService.requireHostAccess(authorization, requireHost(sessionId));
        return relayService.connectMobileSession(sessionId, lastEventId);
    }

    @PostMapping("/sessions/{sessionId}/messages")
    public ApiBaseRet<?> message(@RequestHeader("Authorization") String authorization,
                                 @RequestHeader("Idempotency-Key") String idempotencyKey,
                                 @PathVariable String sessionId,
                                 @RequestBody RemoteProtocol.SendMessageRequest request) throws Exception {
        return command(authorization, idempotencyKey, sessionId, "session.message", objectMapper.valueToTree(request));
    }

    @PostMapping("/sessions/{sessionId}/approvals/{approvalId}")
    public ApiBaseRet<?> approval(@RequestHeader("Authorization") String authorization,
                                  @RequestHeader("Idempotency-Key") String idempotencyKey,
                                  @PathVariable String sessionId,
                                  @PathVariable String approvalId,
                                  @RequestBody RemoteProtocol.ApprovalResolutionRequest request) throws Exception {
        JsonNode payload = objectMapper.valueToTree(request).deepCopy();
        ((com.fasterxml.jackson.databind.node.ObjectNode) payload).put("approvalId", approvalId);
        return command(authorization, idempotencyKey, sessionId, "approval.resolve", payload);
    }

    @PostMapping("/sessions/{sessionId}/questions/{questionId}/reply")
    public ApiBaseRet<?> question(@RequestHeader("Authorization") String authorization,
                                  @RequestHeader("Idempotency-Key") String idempotencyKey,
                                  @PathVariable String sessionId,
                                  @PathVariable String questionId,
                                  @RequestBody RemoteProtocol.QuestionReplyRequest request) throws Exception {
        JsonNode payload = objectMapper.valueToTree(request).deepCopy();
        ((com.fasterxml.jackson.databind.node.ObjectNode) payload).put("questionId", questionId);
        return command(authorization, idempotencyKey, sessionId, "question.reply", payload);
    }

    @PostMapping("/sessions/{sessionId}/cancel")
    public ApiBaseRet<?> cancel(@RequestHeader("Authorization") String authorization,
                                @RequestHeader("Idempotency-Key") String idempotencyKey,
                                @PathVariable String sessionId) throws Exception {
        return command(authorization, idempotencyKey, sessionId, "turn.cancel", empty());
    }

    @PostMapping("/sessions/{sessionId}/retry")
    public ApiBaseRet<?> retry(@RequestHeader("Authorization") String authorization,
                               @RequestHeader("Idempotency-Key") String idempotencyKey,
                               @PathVariable String sessionId) throws Exception {
        return command(authorization, idempotencyKey, sessionId, "turn.retry", empty());
    }

    @GetMapping("/sync")
    public ApiBaseRet<?> sync(@RequestHeader("Authorization") String authorization) {
        authService.requireMobile(authorization);
        List<RemoteProtocol.HostDto> hosts = relayService.hosts().stream()
                .filter(host -> authService.isPairedWith(authorization, host.id())).toList();
        return ApiBaseRet.success(new RemoteProtocol.SyncData(UUID.randomUUID().toString(), false, hosts, List.of(), List.of(), false));
    }

    private ApiBaseRet<?> command(String authorization, String idempotencyKey, String sessionId, String command, JsonNode payload) throws Exception {
        authService.requireMobile(authorization);
        String hostId = requireHost(sessionId);
        authService.requireHostAccess(authorization, hostId);
        RemoteProtocol.DesktopResponse response = relayService.request(hostId, sessionId, command, payload, idempotencyKey);
        return responseEnvelope(response, RemoteProtocol.CommandAcceptedData.class);
    }

    private <T> ApiBaseRet<?> responseEnvelope(RemoteProtocol.DesktopResponse response, Class<T> type) {
        if (!response.success()) {
            throw new IllegalStateException(response.message() == null ? "desktop_request_failed" : response.message());
        }
        T data = response.payload() == null ? null : objectMapper.convertValue(response.payload(), type);
        return ApiBaseRet.success(data);
    }

    private String requireHost(String sessionId) {
        String host = relayService.hostForSession(sessionId);
        if (host == null) {
            throw new IllegalStateException("session_not_found");
        }
        return host;
    }

    @ExceptionHandler(TimeoutException.class)
    public ResponseEntity<ApiBaseRet<?>> timeout(TimeoutException error) {
        return ResponseEntity.status(HttpStatus.GATEWAY_TIMEOUT).body(ApiBaseRet.error(50400, "desktop_timeout"));
    }

    private static JsonNode empty() {
        return JsonNodeFactory.instance.objectNode();
    }

    private static String text(JsonNode node, String field) {
        if (node == null || node.get(field) == null || node.get(field).asText().isBlank()) {
            return UUID.randomUUID().toString();
        }
        return node.get(field).asText();
    }

    private record Health(String status, Instant serverTime) {
    }
}
