package ai.suncode.service;

import ai.suncode.message.remote.RemoteProtocol;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.JsonNodeFactory;
import org.springframework.http.MediaType;
import org.springframework.stereotype.Service;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

import java.io.IOException;
import java.time.Instant;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicLong;

/** Single-node relay state. The interfaces are intentionally kept behind this service for a later shared store. */
@Service
public class RemoteRelayService {
    public static final long REQUEST_TIMEOUT_SECONDS = 15;
    private static final int REPLAY_LIMIT = 256;

    private final ObjectMapper objectMapper;
    private final ConcurrentHashMap<String, DesktopConnection> desktopConnections = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, CopyOnWriteArrayList<MobileConnection>> mobileConnections = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, CompletableFuture<RemoteProtocol.DesktopResponse>> pending = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, RemoteProtocol.DesktopResponse> idempotentResults = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, String> sessionHosts = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, HostEvents> hostEvents = new ConcurrentHashMap<>();

    public RemoteRelayService(ObjectMapper objectMapper) {
        this.objectMapper = objectMapper;
    }

    public SseEmitter connectDesktop(String hostId, String token) {
        requireText(hostId, "X-Host-Id is required");
        DesktopConnection connection = new DesktopConnection(hostId, token, new SseEmitter(0L));
        DesktopConnection previous = desktopConnections.put(hostId, connection);
        if (previous != null) {
            previous.close();
        }
        connection.emitter().onCompletion(() -> desktopConnections.remove(hostId, connection));
        connection.emitter().onTimeout(() -> desktopConnections.remove(hostId, connection));
        connection.emitter().onError(error -> desktopConnections.remove(hostId, connection));
        send(connection.emitter(), "desktop.connected", hostId, JsonNodeFactory.instance.objectNode().put("hostId", hostId));
        return connection.emitter();
    }

    public void requireDesktopConnection(String hostId, String token) {
        DesktopConnection connection = desktopConnections.get(hostId);
        if (connection == null || !connection.token().equals(token)) {
            throw new SecurityException("desktop connection is not registered");
        }
    }

    public RemoteProtocol.DesktopResponse request(
            String hostId,
            String sessionId,
            String command,
            JsonNode payload,
            String idempotencyKey) throws TimeoutException, InterruptedException {
        DesktopConnection connection = desktopConnections.get(hostId);
        if (connection == null) {
            throw new IllegalStateException("desktop_unavailable");
        }
        String idempotencyId = idempotencyKey == null || idempotencyKey.isBlank()
                ? null : hostId + ":" + command + ":" + idempotencyKey;
        if (idempotencyId != null) {
            RemoteProtocol.DesktopResponse previous = idempotentResults.get(idempotencyId);
            if (previous != null) {
                return previous;
            }
        }
        String requestId = UUID.randomUUID().toString();
        CompletableFuture<RemoteProtocol.DesktopResponse> future = new CompletableFuture<>();
        pending.put(requestId, future);
        try {
            RemoteProtocol.DesktopCommand commandEnvelope = new RemoteProtocol.DesktopCommand(
                    requestId, hostId, sessionId, command, payload == null ? JsonNodeFactory.instance.objectNode() : payload);
            send(connection.emitter(), "desktop.command", requestId, objectMapper.valueToTree(commandEnvelope));
            try {
                RemoteProtocol.DesktopResponse response = future.get(REQUEST_TIMEOUT_SECONDS, TimeUnit.SECONDS);
                if (idempotencyId != null) {
                    idempotentResults.putIfAbsent(idempotencyId, response);
                }
                return response;
            } catch (ExecutionException error) {
                throw new IllegalStateException("desktop_request_failed", error.getCause());
            }
        } catch (TimeoutException error) {
            throw error;
        } finally {
            pending.remove(requestId);
        }
    }

    public void complete(RemoteProtocol.DesktopResponse response) {
        if (response == null || response.requestId() == null) {
            throw new IllegalArgumentException("requestId is required");
        }
        CompletableFuture<RemoteProtocol.DesktopResponse> future = pending.get(response.requestId());
        if (future == null) {
            throw new IllegalStateException("request_expired");
        }
        if (response.sessionId() != null) {
            sessionHosts.put(response.sessionId(), response.hostId());
        }
        future.complete(response);
    }

    public RemoteProtocol.MobileEvent publish(RemoteProtocol.DesktopEvent event) {
        requireText(event.hostId(), "hostId is required");
        HostEvents state = hostEvents.computeIfAbsent(event.hostId(), ignored -> new HostEvents());
        long sequence = state.sequence.incrementAndGet();
        String eventId = event.hostId() + ":" + sequence;
        RemoteProtocol.MobileEvent mobileEvent = new RemoteProtocol.MobileEvent(
                eventId,
                sequence,
                event.hostId(),
                event.sessionId(),
                event.requestId(),
                event.eventType(),
                event.occurredAt() == null ? Instant.now() : event.occurredAt(),
                event.payload() == null ? JsonNodeFactory.instance.objectNode() : event.payload());
        synchronized (state.replay) {
            state.replay.addLast(mobileEvent);
            while (state.replay.size() > REPLAY_LIMIT) {
                state.replay.removeFirst();
            }
        }
        if (event.sessionId() != null) {
            sessionHosts.put(event.sessionId(), event.hostId());
            for (MobileConnection mobile : mobileConnections.getOrDefault(event.sessionId(), new CopyOnWriteArrayList<>())) {
                send(mobile.emitter(), "agent.event", mobileEvent.eventId(), objectMapper.valueToTree(mobileEvent));
            }
        }
        return mobileEvent;
    }

    public SseEmitter connectMobileSession(String sessionId, String lastEventId) throws TimeoutException, InterruptedException {
        String hostId = sessionHosts.get(sessionId);
        if (hostId == null) {
            throw new IllegalStateException("session_not_found");
        }
        MobileConnection connection = new MobileConnection(sessionId, new SseEmitter(0L));
        mobileConnections.computeIfAbsent(sessionId, ignored -> new CopyOnWriteArrayList<>()).add(connection);
        connection.emitter().onCompletion(() -> removeMobile(connection));
        connection.emitter().onTimeout(() -> removeMobile(connection));
        connection.emitter().onError(error -> removeMobile(connection));
        try {
            RemoteProtocol.DesktopResponse snapshot = request(hostId, sessionId, "session.get", JsonNodeFactory.instance.objectNode(), null);
            if (!snapshot.success()) {
                throw new IllegalStateException(snapshot.message() == null ? "desktop_request_failed" : snapshot.message());
            }
            HostEvents state = hostEvents.computeIfAbsent(hostId, ignored -> new HostEvents());
            long snapshotSequence = state.sequence.get();
            String snapshotId = hostId + ":" + snapshotSequence;
            RemoteProtocol.SessionSnapshot envelope = new RemoteProtocol.SessionSnapshot(
                    snapshotId, sessionId, snapshotSequence, snapshotSequence,
                    snapshot.payload() == null ? JsonNodeFactory.instance.objectNode() : snapshot.payload());
            send(connection.emitter(), "session.snapshot", snapshotId, objectMapper.valueToTree(envelope));
            replayAfter(connection.emitter(), state, lastEventId, sessionId);
            return connection.emitter();
        } catch (RuntimeException | TimeoutException | InterruptedException error) {
            removeMobile(connection);
            connection.close();
            throw error;
        }
    }

    public List<RemoteProtocol.HostDto> hosts() {
        List<RemoteProtocol.HostDto> result = new ArrayList<>();
        for (Map.Entry<String, DesktopConnection> entry : desktopConnections.entrySet()) {
            result.add(new RemoteProtocol.HostDto(entry.getKey(), entry.getKey(), "", "connected", 0, 0, Instant.now(), null, null));
        }
        return result;
    }

    public String hostForSession(String sessionId) {
        return sessionHosts.get(sessionId);
    }

    public void rememberSession(String sessionId, String hostId) {
        if (sessionId != null && hostId != null) {
            sessionHosts.put(sessionId, hostId);
        }
    }

    private void replayAfter(SseEmitter emitter, HostEvents state, String lastEventId, String sessionId) {
        long cursor = parseSequence(lastEventId);
        synchronized (state.replay) {
            if (!state.replay.isEmpty() && cursor > 0 && cursor < state.replay.peekFirst().sequence() - 1) {
                throw new IllegalStateException("cursor_expired");
            }
            for (RemoteProtocol.MobileEvent event : state.replay) {
                if (event.sequence() > cursor && sessionId.equals(event.sessionId())) {
                    send(emitter, "agent.event", event.eventId(), objectMapper.valueToTree(event));
                }
            }
        }
    }

    private void removeMobile(MobileConnection connection) {
        CopyOnWriteArrayList<MobileConnection> connections = mobileConnections.get(connection.sessionId());
        if (connections != null) {
            connections.remove(connection);
            if (connections.isEmpty()) {
                mobileConnections.remove(connection.sessionId(), connections);
            }
        }
    }

    private void send(SseEmitter emitter, String event, String id, JsonNode data) {
        try {
            emitter.send(SseEmitter.event().name(event).id(id).data(data, MediaType.APPLICATION_JSON));
        } catch (IOException | IllegalStateException ignored) {
            emitter.completeWithError(ignored);
        }
    }

    private static long parseSequence(String eventId) {
        if (eventId == null || eventId.isBlank()) {
            return 0;
        }
        try {
            return Long.parseLong(eventId.substring(eventId.lastIndexOf(':') + 1));
        } catch (RuntimeException ignored) {
            return 0;
        }
    }

    private static void requireText(String value, String message) {
        if (value == null || value.isBlank()) {
            throw new IllegalArgumentException(message);
        }
    }

    private record DesktopConnection(String hostId, String token, SseEmitter emitter) {
        void close() {
            emitter.complete();
        }
    }

    private record MobileConnection(String sessionId, SseEmitter emitter) {
        void close() {
            emitter.complete();
        }
    }

    private static final class HostEvents {
        private final AtomicLong sequence = new AtomicLong();
        private final ArrayDeque<RemoteProtocol.MobileEvent> replay = new ArrayDeque<>();
    }
}
