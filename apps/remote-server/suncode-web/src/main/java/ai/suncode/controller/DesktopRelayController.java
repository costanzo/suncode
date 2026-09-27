package ai.suncode.controller;

import ai.suncode.message.ApiBaseRet;
import ai.suncode.message.remote.RemoteProtocol;
import ai.suncode.service.RemoteAuthService;
import ai.suncode.service.RemoteRelayService;
import com.fasterxml.jackson.databind.JsonNode;
import lombok.RequiredArgsConstructor;
import org.springframework.http.MediaType;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestHeader;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

/** Private Desktop boundary. Desktop receives commands over SSE and completes them over HTTP. */
@RestController
@RequestMapping("/internal/v1/desktop")
@RequiredArgsConstructor
public class DesktopRelayController {
    private final RemoteAuthService authService;
    private final RemoteRelayService relayService;

    @GetMapping(value = "/events", produces = MediaType.TEXT_EVENT_STREAM_VALUE)
    public SseEmitter events(@RequestHeader("X-Host-Id") String hostId,
                             @RequestHeader("Authorization") String authorization) {
        String token = authService.requireDesktop(authorization);
        return relayService.connectDesktop(hostId, token);
    }

    @PostMapping("/pairings")
    public ApiBaseRet<?> createPairing(@RequestHeader("X-Host-Id") String hostId,
                                       @RequestHeader("Authorization") String authorization,
                                       @RequestBody(required = false) PairingRequest request) {
        String token = authService.requireDesktop(authorization);
        relayService.requireDesktopConnection(hostId, token);
        return ApiBaseRet.success(new PairingPayload(authService.createPairing(hostId,
                request == null || request.displayName() == null ? hostId : request.displayName())));
    }

    @PostMapping("/responses")
    public ApiBaseRet<?> response(@RequestHeader("X-Host-Id") String hostId,
                                  @RequestHeader("Authorization") String authorization,
                                  @RequestBody RemoteProtocol.DesktopResponse response) {
        String token = authService.requireDesktop(authorization);
        relayService.requireDesktopConnection(hostId, token);
        if (response.hostId() != null && !hostId.equals(response.hostId())) {
            throw new IllegalArgumentException("hostId does not match the connection");
        }
        relayService.complete(new RemoteProtocol.DesktopResponse(
                response.requestId(), hostId, response.sessionId(), response.success(), response.code(), response.message(), response.payload()));
        return ApiBaseRet.success();
    }

    @PostMapping("/events")
    public ApiBaseRet<?> event(@RequestHeader("X-Host-Id") String hostId,
                               @RequestHeader("Authorization") String authorization,
                               @RequestBody RemoteProtocol.DesktopEvent event) {
        String token = authService.requireDesktop(authorization);
        relayService.requireDesktopConnection(hostId, token);
        RemoteProtocol.DesktopEvent normalized = new RemoteProtocol.DesktopEvent(
                hostId, event.sessionId(), event.requestId(), event.eventType(), event.occurredAt(), event.payload());
        relayService.publish(normalized);
        return ApiBaseRet.success();
    }

    public record PairingRequest(String displayName) {
    }

    public record PairingPayload(String pairingPayload) {
    }
}
