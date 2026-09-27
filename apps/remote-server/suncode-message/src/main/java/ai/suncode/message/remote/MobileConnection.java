package ai.suncode.message.remote;

import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

public record MobileConnection(String sessionId, SseEmitter emitter) {
    public void close() {
        emitter.complete();
    }
}
