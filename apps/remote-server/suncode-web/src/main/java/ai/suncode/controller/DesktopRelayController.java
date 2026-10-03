package ai.suncode.controller;

import ai.suncode.common.exception.BusinessException;
import ai.suncode.common.utils.MarshallingUtils;
import ai.suncode.message.ApiBaseRet;
import ai.suncode.message.remote.*;
import ai.suncode.common.http.ServiceContext;
import ai.suncode.service.RemoteAuthService;
import ai.suncode.service.RemoteRelayService;
import lombok.RequiredArgsConstructor;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestHeader;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;
import com.fasterxml.jackson.databind.JsonNode;

import static ai.suncode.common.exception.ErrorCode.PARAM_INVALID;

/** Private Desktop boundary. Desktop receives commands over SSE and completes them over HTTP. */
@RestController
@RequestMapping("/v1/desktop")
@RequiredArgsConstructor
public class DesktopRelayController {
    private final RemoteAuthService authService;
    private final RemoteRelayService relayService;

    @GetMapping(value = "/events", produces = MediaType.TEXT_EVENT_STREAM_VALUE)
    public SseEmitter events() {
        ServiceContext context = ServiceContext.current();
        return relayService.connectDesktop(context.hostId(), context.token());
    }

    @PostMapping("/pairings")
    public ResponseEntity<DesktopPairingPayload> createPairing(@RequestBody(required = false) DesktopPairingRequest pairingRequest) {
        return ResponseEntity.ok(authService.createDesktopPairing(pairingRequest));
    }

    @PostMapping("/auth/refresh")
    public ResponseEntity<TokenData> refresh(@RequestBody RefreshTokenRequest request) {
        ServiceContext context = ServiceContext.current();
        return ResponseEntity.ok(authService.refreshDesktop(context.hostId(), request));
    }

    @PostMapping("/snapshot")
    public ResponseEntity<Void> snapshot(@RequestBody String snapshot) {
        relayService.updateSnapshot(ServiceContext.current().hostId(), snapshot);
        return ResponseEntity.noContent().build();
    }

    @PostMapping("/responses")
    public ResponseEntity<Void> response(@RequestBody String body) {
        String hostId = ServiceContext.current().hostId();
        String requestId = ServiceContext.current().requestId();
        relayService.complete(new DesktopResponse(requestId, hostId, body));
        return ResponseEntity.noContent().build();
    }

    @PostMapping("/events")
    public ResponseEntity<Void> event(@RequestBody String body) {
        DesktopEvent event = MarshallingUtils.fromJson(body, DesktopEvent.class);
        String hostId = ServiceContext.current().hostId();
        DesktopEvent normalized = new DesktopEvent(
                hostId, event.sessionId(), event.requestId(), event.eventType(), event.occurredAt(), event.payload());
        relayService.publish(normalized);
        return ResponseEntity.noContent().build();
    }

}
