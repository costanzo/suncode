package ai.suncode.message.remote;

import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

public record DesktopConnection(String hostId, String token, SseEmitter emitter) {
    public void close() {
        emitter.complete();
    }
}
