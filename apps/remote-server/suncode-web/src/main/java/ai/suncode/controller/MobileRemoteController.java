package ai.suncode.controller;

import ai.suncode.common.exception.BusinessException;
import ai.suncode.common.utils.MarshallingUtils;
import ai.suncode.message.ApiBaseRet;
import ai.suncode.message.remote.*;
import ai.suncode.common.http.ServiceContext;
import ai.suncode.service.RemoteAuthService;
import ai.suncode.service.RemoteRelayService;
import lombok.RequiredArgsConstructor;
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

import java.time.Instant;
import java.util.List;
import java.util.UUID;

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
    public ApiBaseRet<?> health() {
        return ApiBaseRet.success(new HealthData("ok", Instant.now()));
    }

    @PostMapping("/pairings/exchange")
    public ApiBaseRet<?> exchange(@RequestBody PairingExchangeRequest request) {
        return ApiBaseRet.success(authService.exchange(request));
    }

    @PostMapping("/auth/refresh")
    public ApiBaseRet<?> refresh(@RequestBody RefreshTokenRequest request) {
        return ApiBaseRet.success(authService.refresh(request));
    }

    @PostMapping("/auth/logout")
    public ResponseEntity<Void> logout() {
        authService.logoutToken(ServiceContext.current().token());
        return ResponseEntity.noContent().build();
    }

    @GetMapping("/hosts/{hostId}")
    public ApiBaseRet<?> host(@PathVariable String hostId) {
        return ApiBaseRet.success(relayService.hosts().stream().filter(host -> host.id().equals(hostId)).findFirst()
                .orElseThrow(() -> new BusinessException(HOST_NOT_FOUND)));
    }

    @GetMapping("/hosts/{hostId}/projects")
    public ApiBaseRet<?> projects(@PathVariable String hostId) {
        DesktopResponse response = relayService.request(hostId, null, "projects.list", empty(), null);
        return responseEnvelope(response, ProjectsData.class);
    }

    @GetMapping("/sessions")
    public ApiBaseRet<?> sessions(@RequestParam(required = false) String hostId,
                                  @RequestParam(required = false) String projectId,
                                  @RequestParam(required = false) String cursor,
                                  @RequestParam(required = false) Integer limit) {
        String selectedHost = hostId;
        if (selectedHost == null || selectedHost.isBlank()) {
            selectedHost = ServiceContext.current().hostId();
            String pairedHostId = selectedHost;
            if (relayService.hosts().stream().noneMatch(host -> host.id().equals(pairedHostId))) {
                return ApiBaseRet.success(new SessionPageData(List.of(), null, false));
            }
        }
        DesktopCommandPayload payload = DesktopCommandPayload.listSessions(projectId, cursor, limit);
        return responseEnvelope(relayService.request(selectedHost, null, "sessions.list", payload, null), SessionPageData.class);
    }

    @PostMapping("/sessions")
    public ResponseEntity<ApiBaseRet<?>> createSession(@RequestHeader("Idempotency-Key") String idempotencyKey,
                                                       @RequestBody CreateSessionRequest request) {
        DesktopResponse response = relayService.request(request.hostId(), null, "session.create", DesktopCommandPayload.createSession(request), idempotencyKey);
        String sessionId = sessionId(response.payload());
        relayService.rememberSession(sessionId, request.hostId());
        return ResponseEntity.status(HttpStatus.CREATED).body(ApiBaseRet.success(new CommandAcceptedData(
                response.requestId(), Instant.now(), sessionId)));
    }

    @GetMapping("/sessions/{sessionId}")
    public ApiBaseRet<?> session(@PathVariable String sessionId) {
        String hostId = requireHost(sessionId);
        return responseEnvelope(relayService.request(hostId, sessionId, "session.get", empty(), null), Object.class);
    }

    @GetMapping(value = "/sessions/{sessionId}/events", produces = MediaType.TEXT_EVENT_STREAM_VALUE)
    public SseEmitter sessionEvents(@PathVariable String sessionId,
                                    @RequestHeader(value = "Last-Event-ID", required = false) String lastEventId) {
        return relayService.connectMobileSession(sessionId, lastEventId);
    }

    @PostMapping("/sessions/{sessionId}/messages")
    public ApiBaseRet<?> message(@RequestHeader("Idempotency-Key") String idempotencyKey,
                                 @PathVariable String sessionId,
                                 @RequestBody SendMessageRequest request) {
        return command(idempotencyKey, sessionId, "session.message", DesktopCommandPayload.sendMessage(request));
    }

    @PostMapping("/sessions/{sessionId}/approvals/{approvalId}")
    public ApiBaseRet<?> approval(@RequestHeader("Idempotency-Key") String idempotencyKey,
                                  @PathVariable String sessionId,
                                  @PathVariable String approvalId,
                                  @RequestBody ApprovalResolutionRequest request) {
        return command(idempotencyKey, sessionId, "approval.resolve", DesktopCommandPayload.resolveApproval(approvalId, request));
    }

    @PostMapping("/sessions/{sessionId}/questions/{questionId}/reply")
    public ApiBaseRet<?> question(@RequestHeader("Idempotency-Key") String idempotencyKey,
                                  @PathVariable String sessionId,
                                  @PathVariable String questionId,
                                  @RequestBody QuestionReplyRequest request) {
        return command(idempotencyKey, sessionId, "question.reply", DesktopCommandPayload.replyQuestion(questionId, request));
    }

    @PostMapping("/sessions/{sessionId}/cancel")
    public ApiBaseRet<?> cancel(@RequestHeader("Idempotency-Key") String idempotencyKey,
                                @PathVariable String sessionId) {
        return command(idempotencyKey, sessionId, "turn.cancel", empty());
    }

    @PostMapping("/sessions/{sessionId}/retry")
    public ApiBaseRet<?> retry(@RequestHeader("Idempotency-Key") String idempotencyKey,
                               @PathVariable String sessionId) {
        return command(idempotencyKey, sessionId, "turn.retry", empty());
    }

    @GetMapping("/sync")
    public ApiBaseRet<?> sync() {
        String hostId = ServiceContext.current().hostId();
        List<HostDto> hosts = relayService.hosts().stream().filter(host -> host.id().equals(hostId)).toList();
        return ApiBaseRet.success(new SyncData(UUID.randomUUID().toString(), false, hosts, List.of(), List.of(), false));
    }

    private ApiBaseRet<?> command(String idempotencyKey, String sessionId, String command, DesktopCommandPayload payload) {
        String hostId = requireHost(sessionId);
        DesktopResponse response = relayService.request(hostId, sessionId, command, payload, idempotencyKey);
        return responseEnvelope(response, CommandAcceptedData.class);
    }

    private <T> ApiBaseRet<?> responseEnvelope(DesktopResponse response, Class<T> type) {
        if (!response.success()) {
            throw new BusinessException(CONFLICT, response.message() == null ? "desktop_request_failed" : response.message());
        }
        T data = response.payload() == null ? null : MarshallingUtils.fromJson(response.payload(), type);
        return ApiBaseRet.success(data);
    }

    private String requireHost(String sessionId) {
        String host = relayService.hostForSession(sessionId);
        if (host == null) {
            throw new BusinessException(SESSION_NOT_FOUND);
        }
        return host;
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
