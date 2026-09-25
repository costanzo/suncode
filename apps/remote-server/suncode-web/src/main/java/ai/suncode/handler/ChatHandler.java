package ai.suncode.handler;


import jakarta.annotation.Nonnull;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;
import org.springframework.web.socket.CloseStatus;
import org.springframework.web.socket.TextMessage;
import org.springframework.web.socket.WebSocketSession;
import org.springframework.web.socket.handler.TextWebSocketHandler;

@Slf4j
@Component
public class ChatHandler extends TextWebSocketHandler {

    @Override
    public void afterConnectionEstablished(@Nonnull WebSocketSession session) throws Exception {
        log.info("Connection established: {}", session);
    }

    @Override
    public void afterConnectionClosed(@Nonnull WebSocketSession session, @Nonnull CloseStatus status) throws Exception {
        log.info("Connection closed: {}", session);
    }

    @Override
    protected void handleTextMessage(@Nonnull WebSocketSession session, @Nonnull TextMessage message) throws Exception {
        log.info("Message received: {}", message);
        final String payload = message.getPayload();
    }
}
