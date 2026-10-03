package ai.suncode.controller;

import ai.suncode.common.exception.BusinessException;
import ai.suncode.common.utils.MarshallingUtils;
import ai.suncode.message.ApiBaseRet;
import ai.suncode.message.remote.*;
import ai.suncode.common.http.ServiceContext;
import ai.suncode.service.RemoteAuthService;
import ai.suncode.service.RemoteRelayService;
import lombok.RequiredArgsConstructor;
import org.apache.el.parser.Token;
import org.springframework.http.HttpStatus;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestHeader;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.JsonNodeFactory;

import java.time.Instant;
import java.util.List;
import java.util.UUID;
import java.util.Map;
import java.util.function.Function;

import static ai.suncode.common.exception.ErrorCode.CONFLICT;
import static ai.suncode.common.exception.ErrorCode.HOST_NOT_FOUND;
import static ai.suncode.common.exception.ErrorCode.SESSION_NOT_FOUND;

@RestController
@RequestMapping("/v1/mobile")
@RequiredArgsConstructor
public class MobileRemoteController {
    private final RemoteAuthService authService;
    private final RemoteRelayService relayService;

    @GetMapping("/health")
    public ResponseEntity<HealthData> health() {
        return ResponseEntity.ok(new HealthData("ok", Instant.now()));
    }

    @PostMapping("/pairings/exchange")
    public ResponseEntity<PairingExchangeData> exchange(@RequestBody PairingExchangeRequest request) {
        String hostId = ServiceContext.current().hostId();
        authService.requirePairingHost(hostId, request == null ? null : request.pairingCode());
        PairingExchangeData data = authService.exchange(request);
        relayService.notifyDesktop(hostId, ServiceContext.current().requestId(), "mobile.pairings.exchange.succeeded",
                java.util.Map.of("pathParam", java.util.Map.of(), "queryParam", java.util.Map.of(), "requestBody", java.util.Map.of("deviceName", request.deviceName())));
        return ResponseEntity.ok(data);
    }

    @PostMapping("/auth/refresh")
    public ResponseEntity<TokenData> refresh(@RequestBody RefreshTokenRequest request) {
        TokenData data = authService.refreshMobile(ServiceContext.current().hostId(), request);
        return ResponseEntity.ok(data);
    }

    @PostMapping("/auth/logout")
    public ResponseEntity<Void> logout() {
        String hostId = ServiceContext.current().hostId();
        authService.logoutToken(ServiceContext.current().token());
        relayService.notifyDesktop(hostId, ServiceContext.current().requestId(), "mobile.auth.logout",
                java.util.Map.of("pathParam", java.util.Map.of(), "queryParam", java.util.Map.of(), "requestBody", java.util.Map.of()));
        return ResponseEntity.noContent().build();
    }

    @GetMapping("/hosts/{hostId}")
    public ResponseEntity<HostDto> host(@PathVariable String hostId) {
        HostDto hostDto = relayService.hosts().stream().filter(host -> host.id().equals(hostId)).findFirst()
                .orElseThrow(() -> new BusinessException(HOST_NOT_FOUND));
        return ResponseEntity.ok(hostDto);
    }

    @GetMapping("/hosts/{hostId}/projects")
    public ResponseEntity<ProjectsData> projects(@PathVariable String hostId) {
        DesktopResponse response = relayService.request(hostId, null, "projects.list", empty(), ServiceContext.current().requestId());
        return ResponseEntity.ok(responseEnvelope(response, ProjectsData.class));
    }

    @GetMapping("/hosts/{hostId}/sessions")
    public ResponseEntity<SessionPageData> sessions(@PathVariable String hostId,
                                  @RequestParam(required = false) String projectId,
                                  @RequestParam(required = false) String cursor,
                                  @RequestParam(required = false) Integer limit) {
        DesktopCommandPayload payload = DesktopCommandPayload.listSessions(projectId, cursor, limit);
        java.util.Map<String, Object> query = new java.util.LinkedHashMap<>();
        if (projectId != null) query.put("projectId", projectId);
        if (cursor != null) query.put("cursor", cursor);
        if (limit != null) query.put("limit", limit);
        return ResponseEntity.ok(responseEnvelope(relayService.request(hostId, null, "sessions.list", payload, ServiceContext.current().requestId(), query), SessionPageData.class));
    }

    @PostMapping("/hosts/{hostId}/sessions")
    public ResponseEntity<ApiBaseRet<?>> createSession(@PathVariable String hostId,
                                                       @RequestParam(required = false) String projectId,
                                                       @RequestBody JsonNode request) {
        Map<String, Object> query = projectId == null ? Map.of() : Map.of("projectId", projectId);
        DesktopResponse response = requestDesktop(hostId, null, "session.create", null, request, query,
                body -> DesktopCommandPayload.createSession(MarshallingUtils.convertValue(body, CreateSessionRequest.class)));
        String sessionId = sessionId(response.payload());
        relayService.rememberSession(sessionId, hostId);
        return ResponseEntity.status(HttpStatus.CREATED).body(ApiBaseRet.success(new CommandAcceptedData(sessionId)));
    }

    @GetMapping("/hosts/{hostId}/sessions/{sessionId}")
    public ResponseEntity<Object> session(@PathVariable String hostId, @PathVariable String sessionId) {
        return ResponseEntity.ok(responseEnvelope(relayService.request(hostId, sessionId, "session.get", empty(), ServiceContext.current().requestId()), Object.class));
    }

    @GetMapping(value = "/hosts/{hostId}/sessions/{sessionId}/events", produces = MediaType.TEXT_EVENT_STREAM_VALUE)
    public SseEmitter sessionEvents(@PathVariable String hostId, @PathVariable String sessionId,
                                    @RequestHeader(value = "Last-Event-ID", required = false) String lastEventId) {
        return relayService.connectMobileSession(hostId, sessionId, lastEventId);
    }

    @PostMapping("/hosts/{hostId}/sessions/{sessionId}/messages")
    public ResponseEntity<CommandAcceptedData> message(@PathVariable String hostId,
                                 @PathVariable String sessionId,
                                 @RequestBody JsonNode request) {
        return ResponseEntity.ok(responseEnvelope(requestDesktop(hostId, sessionId, "session.message", null, request, Map.of(),
                body -> DesktopCommandPayload.sendMessage(MarshallingUtils.convertValue(body, SendMessageRequest.class))), CommandAcceptedData.class));
    }

    @PostMapping("/hosts/{hostId}/sessions/{sessionId}/approvals/{approvalId}")
    public ResponseEntity<CommandAcceptedData> approval(@PathVariable String hostId,
                                  @PathVariable String sessionId,
                                  @PathVariable String approvalId,
                                  @RequestBody JsonNode request) {
        return ResponseEntity.ok(responseEnvelope(requestDesktop(hostId, sessionId, "approval.resolve", approvalId, request, Map.of(),
                body -> DesktopCommandPayload.resolveApproval(approvalId, MarshallingUtils.convertValue(body, ApprovalResolutionRequest.class))), CommandAcceptedData.class));
    }

    @PostMapping("/hosts/{hostId}/sessions/{sessionId}/questions/{questionId}/reply")
    public ResponseEntity<CommandAcceptedData> question(@PathVariable String hostId,
                                  @PathVariable String sessionId,
                                  @PathVariable String questionId,
                                  @RequestBody JsonNode request) {
        return ResponseEntity.ok(responseEnvelope(requestDesktop(hostId, sessionId, "question.reply", questionId, request, Map.of(),
                body -> DesktopCommandPayload.replyQuestion(questionId, MarshallingUtils.convertValue(body, QuestionReplyRequest.class))), CommandAcceptedData.class));
    }

    @PostMapping("/hosts/{hostId}/sessions/{sessionId}/cancel")
    public ResponseEntity<Void> cancel(@PathVariable String hostId,
                                @PathVariable String sessionId) {
        command(hostId, sessionId, "turn.cancel", empty());
        return ResponseEntity.noContent().build();
    }

    @PostMapping("/hosts/{hostId}/sessions/{sessionId}/retry")
    public ResponseEntity<Void> retry(@PathVariable String hostId,
                               @PathVariable String sessionId) {
        command(hostId, sessionId, "turn.retry", empty());
        return ResponseEntity.noContent().build();
    }

    @GetMapping("/hosts/{hostId}/sync")
    public ResponseEntity<SyncData> sync(@PathVariable String hostId) {
        String snapshot = relayService.snapshot(hostId);
        if (snapshot == null) {
            return ResponseEntity.ok().build();
        } else {
            return ResponseEntity.ok(MarshallingUtils.fromJson(snapshot, SyncData.class));
        }
    }

    private void command(String hostId, String sessionId, String command, DesktopCommandPayload payload) {
        relayService.request(hostId, sessionId, command, payload, ServiceContext.current().requestId());
    }

    private DesktopResponse requestDesktop(String hostId, String sessionId, String command, String routeId,
                                           JsonNode request, Map<String, Object> query,
                                           Function<JsonNode, DesktopCommandPayload> plaintextMapper) {
        String encrypted = encryptedPayload(request);
        String requestId = ServiceContext.current().requestId();
        if (encrypted != null) {
            return relayService.requestEncrypted(hostId, sessionId, command, routeId, requestId, query, encrypted);
        }
        JsonNode body = request == null ? JsonNodeFactory.instance.objectNode() : request;
        return relayService.request(hostId, sessionId, command, plaintextMapper.apply(body), requestId, query);
    }

    private static String encryptedPayload(JsonNode request) {
        if (request == null || !request.isObject()) return null;
        JsonNode encrypted = request.get("encPayload");
        return encrypted != null && encrypted.isTextual() && !encrypted.textValue().isBlank()
                ? encrypted.textValue() : null;
    }

    private <T> T responseEnvelope(DesktopResponse response, Class<T> type) {
        return response.payload() == null ? null : MarshallingUtils.fromJson(response.payload(), type);
    }

    private static DesktopCommandPayload empty() {
        return new DesktopCommandPayload();
    }

    private static String sessionId(String payload) {
        CommandAcceptedData created = payload == null ? null : MarshallingUtils.fromJson(payload, CommandAcceptedData.class);
        if (created == null || created.sessionId() == null || created.sessionId().isBlank()) {
            return UUID.randomUUID().toString();
        }
        return created.sessionId();
    }

}
