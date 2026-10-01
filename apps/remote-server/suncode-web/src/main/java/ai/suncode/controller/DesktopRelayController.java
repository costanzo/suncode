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
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
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
    public ApiBaseRet<?> createPairing(@RequestBody(required = false) DesktopPairingRequest pairingRequest) {
        return ApiBaseRet.success(authService.createDesktopPairing(pairingRequest));
    }

    @PostMapping("/snapshot")
    public ApiBaseRet<?> snapshot(@RequestBody JsonNode snapshot) {
        relayService.updateSnapshot(ServiceContext.current().hostId(), snapshot);
        return ApiBaseRet.success();
    }

    @PostMapping("/responses")
    public ApiBaseRet<?> response(@RequestBody String body) {
        DesktopResponse response = MarshallingUtils.fromJson(body, DesktopResponse.class);
        String hostId = ServiceContext.current().hostId();
        if (response.hostId() != null && !hostId.equals(response.hostId())) {
            throw new BusinessException(PARAM_INVALID, "hostId does not match the connection");
        }
        relayService.complete(new DesktopResponse(
                response.requestId(), hostId, response.sessionId(), response.success(), response.code(), response.message(), response.payload()));
        return ApiBaseRet.success();
    }

    @PostMapping("/events")
    public ApiBaseRet<?> event(@RequestBody String body) {
        DesktopEvent event = MarshallingUtils.fromJson(body, DesktopEvent.class);
        String hostId = ServiceContext.current().hostId();
        DesktopEvent normalized = new DesktopEvent(
                hostId, event.sessionId(), event.requestId(), event.eventType(), event.occurredAt(), event.payload());
        relayService.publish(normalized);
        return ApiBaseRet.success();
    }

}
