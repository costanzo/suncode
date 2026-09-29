package ai.suncode.scheduler;

import ai.suncode.service.RemoteRelayService;
import jakarta.annotation.PostConstruct;
import jakarta.annotation.PreDestroy;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@Component
public class SseHeartbeatScheduler {
    private static final long HEARTBEAT_INTERVAL_MILLIS = 15_000L;
    private final RemoteRelayService relayService;
    private final long intervalMillis;

    private ScheduledExecutorService executorService;
    private final AtomicReference<ScheduledFuture<?>> scheduledTask = new AtomicReference<>();

    @Autowired
    public SseHeartbeatScheduler(RemoteRelayService relayService) {
        this.relayService = relayService;
        this.intervalMillis = HEARTBEAT_INTERVAL_MILLIS;
    }

    @PostConstruct
    public void start() {
        ThreadFactory factory = new ThreadFactory() {
            private final AtomicInteger counter = new AtomicInteger();
            @Override
            public Thread newThread(Runnable r) {
                Thread thread = new Thread(r, "sse-heartbeat-" + counter.incrementAndGet());
                thread.setDaemon(true);
                return thread;
            }
        };
        executorService = Executors.newSingleThreadScheduledExecutor(factory);
        ScheduledFuture<?> task = executorService.scheduleAtFixedRate(this::runHeartbeat,
                intervalMillis, intervalMillis, TimeUnit.MILLISECONDS);
        scheduledTask.set(task);
        log.info("SSE Heartbeat Scheduler started with interval: {} ms", intervalMillis);
    }

    @PreDestroy
    public void stop() {
        ScheduledFuture<?> task = scheduledTask.get();
        if (task != null) {
            task.cancel(true);
        }
        if (executorService != null) {
            executorService.shutdownNow();
        }
    }

    private void runHeartbeat() {
        try {
            relayService.heartbeat();
        } catch (Exception e) {
            log.error("Error while sending SSE heartbeat", e);
        }
    }
}
