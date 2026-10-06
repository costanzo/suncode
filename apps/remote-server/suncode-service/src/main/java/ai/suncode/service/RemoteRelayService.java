package ai.suncode.service;

import ai.suncode.common.exception.BusinessException;
import ai.suncode.common.utils.DesktopPayloadMapper;
import ai.suncode.common.utils.MarshallingUtils;
import ai.suncode.message.remote.*;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.ContextClosedEvent;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Service;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

import java.io.IOException;
import java.time.Instant;
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

import static ai.suncode.common.exception.ErrorCode.PARAM_INVALID;
import static ai.suncode.common.exception.ErrorCode.UNAUTHORIZED;
import static ai.suncode.common.exception.ErrorCode.DESKTOP_UNAVAILABLE;
import static ai.suncode.common.exception.ErrorCode.DESKTOP_REQUEST_FAILED;
import static ai.suncode.common.exception.ErrorCode.DESKTOP_TIMEOUT;
import static ai.suncode.common.exception.ErrorCode.REQUEST_EXPIRED;
import static ai.suncode.common.exception.ErrorCode.SESSION_NOT_FOUND;
import static ai.suncode.common.exception.ErrorCode.CURSOR_EXPIRED;
import static ai.suncode.common.exception.ErrorCode.INTERNAL_ERROR;

/** Single-node relay state. The interfaces are intentionally kept behind this service for a later shared store. */
@Service
@Slf4j
public class RemoteRelayService {
    public static final long REQUEST_TIMEOUT_SECONDS = 15;
    private static final int REPLAY_LIMIT = 256;
    private static final String PING_EVENT = "ping";

    private final ConcurrentHashMap<String, DesktopConnection> desktopConnections = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, CopyOnWriteArrayList<MobileConnection>> mobileConnections = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, CompletableFuture<DesktopResponse>> pending = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, DesktopResponse> idempotentResults = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, HostEvents> hostEvents = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, String> snapshots = new ConcurrentHashMap<>();
    private final Object connectionLifecycleLock = new Object();
    private volatile boolean shuttingDown;

    public SseEmitter connectDesktop(String hostId, String token) {
        requireText(hostId, "X-Host-Id is required");
        DesktopConnection connection = new DesktopConnection(hostId, token, new SseEmitter(0L));
        DesktopConnection previous;
        synchronized (connectionLifecycleLock) {
            if (shuttingDown) {
                connection.close();
                return connection.emitter();
            }
            previous = desktopConnections.put(hostId, connection);
        }
        if (previous != null) {
            previous.close();
        }
        connection.emitter().onCompletion(() -> desktopConnections.remove(hostId, connection));
        connection.emitter().onTimeout(() -> desktopConnections.remove(hostId, connection));
        connection.emitter().onError(error -> desktopConnections.remove(hostId, connection));
        send(connection.emitter(), "desktop.connected", hostId, Map.of("hostId", hostId));
        return connection.emitter();
    }

    public void requireDesktopConnection(String hostId, String token) {
        DesktopConnection connection = desktopConnections.get(hostId);
        if (connection == null || !connection.token().equals(token)) {
            throw new BusinessException(UNAUTHORIZED, "desktop connection is not registered");
        }
    }

    public DesktopResponse request(
            String hostId,
            String sessionId,
            String command,
            DesktopCommandPayload payload,
            String requestIdHeader) {
        return request(hostId, sessionId, command, payload, requestIdHeader, Map.of());
    }

    public DesktopResponse request(
            String hostId,
            String sessionId,
            String command,
            DesktopCommandPayload payload,
            String requestIdHeader,
            Map<String, Object> queryParam) {
        return requestInternal(hostId, sessionId, command, payload, null, requestIdHeader, queryParam);
    }

    public DesktopResponse request(
            String hostId,
            String sessionId,
            String command,
            DesktopCommandPayload payload,
            String routeId,
            String requestIdHeader,
            Map<String, Object> queryParam) {
        String effectiveRouteId = routeId;
        if (effectiveRouteId == null && payload != null) {
            if (command.startsWith("approval.")) {
                effectiveRouteId = payload.getApprovalId();
            } else if (command.startsWith("question.")) {
                effectiveRouteId = payload.getQuestionId();
            }
        }
        return requestInternal(hostId, sessionId, command, payload, effectiveRouteId, requestIdHeader, queryParam);
    }

    /** Forward an encrypted request body without interpreting its contents. */
    public DesktopResponse requestEncrypted(
            String hostId,
            String sessionId,
            String command,
            String requestIdHeader,
            Map<String, Object> queryParam,
            String encPayload) {
        return requestRaw(hostId, sessionId, command, null, requestIdHeader, queryParam,
                MarshallingUtils.toJson(Map.of("encPayload", encPayload)));
    }

    public DesktopResponse requestEncrypted(
            String hostId,
            String sessionId,
            String command,
            String routeId,
            String requestIdHeader,
            Map<String, Object> queryParam,
            String encPayload) {
        return requestRaw(hostId, sessionId, command, routeId, requestIdHeader, queryParam,
                MarshallingUtils.toJson(Map.of("encPayload", encPayload)));
    }

    private DesktopResponse requestInternal(
            String hostId,
            String sessionId,
            String command,
            DesktopCommandPayload payload,
            String routeId,
            String requestId,
            Map<String, Object> queryParam) {
        return requestRaw(hostId, sessionId, command, routeId, requestId, queryParam,
                MarshallingUtils.toJson(payload == null ? new DesktopCommandPayload() : payload));
    }

    public DesktopResponse requestRaw(
            String hostId,
            String sessionId,
            String command,
            String routeId,
            String requestId,
            Map<String, Object> queryParam,
            String requestBody) {
        DesktopConnection connection = desktopConnections.get(hostId);
        if (connection == null) {
            throw new BusinessException(DESKTOP_UNAVAILABLE);
        }
        // Internal relay operations, such as the initial Session snapshot, do
        // not originate from a Mobile HTTP request. They still need a unique
        // SSE id so Spring can serialize the event and Desktop can correlate
        // the response.
        String effectiveRequestId = requestId == null || requestId.isBlank()
                ? UUID.randomUUID().toString()
                : requestId;
        String idempotencyId = hostId + ":" + effectiveRequestId;
        DesktopResponse previous = idempotentResults.get(idempotencyId);
        if (previous != null) {
            return previous;
        }
        CompletableFuture<DesktopResponse> future = new CompletableFuture<>();
        synchronized (connectionLifecycleLock) {
            if (shuttingDown) {
                throw new BusinessException(DESKTOP_UNAVAILABLE, "sever is shutting down");
            }
            pending.put(idempotencyId, future);
        }
        try {
            Map<String, Object> path = new java.util.LinkedHashMap<>();
            if (hostId != null) path.put("hostId", hostId);
            if (sessionId != null) path.put("sessionId", sessionId);
            if (command.startsWith("approval.") && routeId != null) {
                path.put("approvalId", routeId);
            }
            if (command.startsWith("question.") && routeId != null) {
                path.put("questionId", routeId);
            }
            DesktopMobileHttpRequest requestEnvelope = new DesktopMobileHttpRequest(
                    path, queryParam == null ? Map.of() : queryParam,
                    requestBody == null || requestBody.isBlank() ? "{}" : requestBody);
            send(connection.emitter(), mobileEventName(command), effectiveRequestId, requestEnvelope);
            try {
                DesktopResponse response = future.get(REQUEST_TIMEOUT_SECONDS, TimeUnit.SECONDS);
                idempotentResults.putIfAbsent(idempotencyId, response);
                return response;
            } catch (ExecutionException error) {
                if (error.getCause() instanceof BusinessException businessError) {
                    throw businessError;
                }
                throw new BusinessException(DESKTOP_REQUEST_FAILED, "desktop_request_failed", error.getCause());
            }
        } catch (TimeoutException error) {
            throw new BusinessException(DESKTOP_TIMEOUT, "desktop_timeout", error);
        } catch (InterruptedException error) {
            Thread.currentThread().interrupt();
            throw new BusinessException(DESKTOP_REQUEST_FAILED, "desktop_request_interrupted", error);
        } finally {
            pending.remove(idempotencyId);
        }
    }

    public void complete(DesktopResponse response) {
        if (response == null || response.requestId() == null || response.hostId() == null) {
            throw new BusinessException(PARAM_INVALID, "requestId/hostId is required");
        }
        String idempotencyId = response.hostId() + ":" + response.requestId();
        CompletableFuture<DesktopResponse> future = pending.get(idempotencyId);
        if (future == null) {
            throw new BusinessException(REQUEST_EXPIRED);
        }
        // Success bodies are the bare payload; only a {code, message} body signals a Desktop failure.
        DesktopPayloadMapper.error(response.payload()).ifPresentOrElse(
                error -> future.completeExceptionally(new BusinessException(DESKTOP_REQUEST_FAILED,
                        error.message() == null || error.message().isBlank() ? "desktop_request_failed" : error.message())),
                () -> future.complete(response));
    }

    public MobileEvent publish(DesktopEvent event) {
        requireText(event.hostId(), "hostId is required");
        HostEvents state = hostEvents.computeIfAbsent(event.hostId(), ignored -> new HostEvents());
        long sequence = state.sequence().incrementAndGet();
        String eventId = event.hostId() + ":" + sequence;
        MobileEvent mobileEvent = new MobileEvent(
                eventId,
                sequence,
                event.hostId(),
                event.sessionId(),
                event.requestId(),
                event.eventType(),
                event.occurredAt(),
                event.payload() == null ? new EventPayload() : event.payload());
        synchronized (state.replay()) {
            state.replay().addLast(mobileEvent);
            while (state.replay().size() > REPLAY_LIMIT) {
                state.replay().removeFirst();
            }
        }
        if (event.sessionId() != null) {
            for (MobileConnection mobile : mobileConnections.getOrDefault(event.sessionId(), new CopyOnWriteArrayList<>())) {
                send(mobile.emitter(), "agent.event", mobileEvent.eventId(), mobileEvent);
            }
        }
        return mobileEvent;
    }

    public SseEmitter connectMobileSession(String hostId, String sessionId, String lastEventId,
                                           String requestId) {
        if (hostId == null || sessionId == null) throw new BusinessException(SESSION_NOT_FOUND);
        MobileConnection connection = new MobileConnection(sessionId, new SseEmitter(0L));
        synchronized (connectionLifecycleLock) {
            if (shuttingDown) {
                connection.close();
                return connection.emitter();
            }
            mobileConnections.computeIfAbsent(sessionId, ignored -> new CopyOnWriteArrayList<>()).add(connection);
        }
        connection.emitter().onCompletion(() -> removeMobile(connection));
        connection.emitter().onTimeout(() -> removeMobile(connection));
        connection.emitter().onError(error -> removeMobile(connection));
        try {
            DesktopResponse snapshot = request(hostId, sessionId, "session.get", new DesktopCommandPayload(), requestId);
            HostEvents state = hostEvents.computeIfAbsent(hostId, ignored -> new HostEvents());
            long snapshotSequence = state.sequence().get();
            String snapshotId = hostId + ":" + snapshotSequence;
            SessionSnapshot envelope = new SessionSnapshot(
                    snapshotId, sessionId, snapshotSequence, snapshotSequence,
                    snapshot.payload() == null ? "{}" : snapshot.payload());
            send(connection.emitter(), "session.snapshot", snapshotId, envelope);
            replayAfter(connection.emitter(), state, lastEventId, sessionId);
            return connection.emitter();
        } catch (BusinessException error) {
            removeMobile(connection);
            connection.close();
            throw error;
        } catch (RuntimeException error) {
            removeMobile(connection);
            connection.close();
            throw new BusinessException(INTERNAL_ERROR, "session stream failed", error);
        }
    }

    @EventListener(ContextClosedEvent.class)
    public void onCloseContext() {
        List<DesktopConnection> desktops;
        List<MobileConnection> mobiles = new ArrayList<>();
        List<CompletableFuture<DesktopResponse>> pendingFutures;
        synchronized (connectionLifecycleLock) {
            shuttingDown = true;
            desktops = new ArrayList<>(desktopConnections.values());
            desktopConnections.clear();
            for (CopyOnWriteArrayList<MobileConnection> connections : mobileConnections.values()) {
                mobiles.addAll(connections);
            }
            mobileConnections.clear();
            pendingFutures = new ArrayList<>(pending.values());
            pending.clear();
        }
        desktops.forEach(DesktopConnection::close);
        mobiles.forEach(MobileConnection::close);
        pendingFutures.forEach(future -> future.completeExceptionally(new BusinessException(DESKTOP_UNAVAILABLE, "server is shutting down")));
        log.info("Closed {} desktop connections, {} mobile connections, and {} pending requests due to server shutdown", desktops.size(), mobiles.size(), pendingFutures.size());
    }

    public List<HostDto> hosts() {
        List<HostDto> result = new ArrayList<>();
        for (Map.Entry<String, DesktopConnection> entry : desktopConnections.entrySet()) {
            result.add(new HostDto(entry.getKey(), entry.getKey(), "connected", Instant.now(), null));
        }
        return result;
    }

    public void updateSnapshot(String hostId, String snapshot) {
        requireText(hostId, "hostId is required");
        snapshots.put(hostId, snapshot);
    }

    public String snapshot(String hostId) {
        return snapshots.get(hostId);
    }

    public void notifyDesktop(String hostId, String requestId, String eventType, Object body) {
        DesktopConnection connection = desktopConnections.get(hostId);
        if (connection != null) {
            send(connection.emitter(), eventType, requestId == null ? UUID.randomUUID().toString() : requestId,
                    new DesktopMobileHttpRequest(Map.of(), Map.of(),
                            MarshallingUtils.toJson(body == null ? Map.of() : body)));
        }
    }

    private void replayAfter(SseEmitter emitter, HostEvents state, String lastEventId, String sessionId) {
        long cursor = parseSequence(lastEventId);
        synchronized (state.replay()) {
            if (!state.replay().isEmpty() && cursor > 0 && cursor < state.replay().peekFirst().sequence() - 1) {
                throw new BusinessException(CURSOR_EXPIRED);
            }
            for (MobileEvent event : state.replay()) {
                if (event.sequence() > cursor && sessionId.equals(event.sessionId())) {
                    send(emitter, "agent.event", event.eventId(), event);
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

    private void send(SseEmitter emitter, String event, String id, Object data) {
        try {
            if (data instanceof DesktopMobileHttpRequest request) {
                String routingJson = MarshallingUtils.toJson(Map.of(
                        "pathParam", request.pathParam() == null ? Map.of() : request.pathParam(),
                        "queryParam", request.queryParam() == null ? Map.of() : request.queryParam()));
                String requestBody = request.requestBody() == null || request.requestBody().isBlank()
                        ? "{}" : request.requestBody();
                log.info("SSE SEND ==> event: {}, id: {}, routingData: {}, bodyData: {}",
                        event, id, routingJson, requestBody);
                emitter.send(SseEmitter.event().name(event).id(id).data(routingJson).data(requestBody));
                return;
            }
            String json = MarshallingUtils.toJson(data);
            log.info("SSE SEND ==> event: {}, id: {}, data: {}", event, id, json);
            emitter.send(SseEmitter.event().name(event).id(id).data(json));
        } catch (Exception e) {
            log.warn("SSE send failed, event={}, id={}", event, id, e);
            emitter.completeWithError(e);
        }
    }

    public void heartbeat() {
        int desktopCount = desktopConnections.size();
        for (DesktopConnection connection : desktopConnections.values()) {
            sendComment(connection.emitter());
        }
        int mobileCount = 0;
        for (CopyOnWriteArrayList<MobileConnection> connections : mobileConnections.values()) {
            for (MobileConnection connection : connections) {
                sendComment(connection.emitter());
                mobileCount++;
            }
        }
        log.info("Heartbeat sent to {} desktop connections and {} mobile connections", desktopCount, mobileCount);
    }

    private void sendComment(SseEmitter emitter) {
        try {
            emitter.send(SseEmitter.event().comment(PING_EVENT));
        } catch (IOException | IllegalStateException ignored) {
            try {
                emitter.complete();
            } catch (RuntimeException e) {
                // Ignore if the emitter is already completed
            }
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
            throw new BusinessException(PARAM_INVALID, message);
        }
    }

    private static String mobileEventName(String command) {
        return switch (command) {
            case "projects.list" -> "mobile.projects.list";
            case "sessions.list" -> "mobile.sessions.list";
            case "session.create" -> "mobile.sessions.create";
            case "session.get" -> "mobile.sessions.get";
            case "session.message", "session.send_message" -> "mobile.sessions.messages.send";
            case "approval.resolve" -> "mobile.sessions.approvals.resolve";
            case "question.reply" -> "mobile.sessions.questions.reply";
            case "turn.cancel", "session.cancel" -> "mobile.sessions.cancel";
            case "turn.retry", "session.retry" -> "mobile.sessions.retry";
            default -> "mobile." + command.replace('.', '_');
        };
    }

}
