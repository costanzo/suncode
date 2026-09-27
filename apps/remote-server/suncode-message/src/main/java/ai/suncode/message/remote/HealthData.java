package ai.suncode.message.remote;

import java.time.Instant;

public record HealthData(String status, Instant serverTime) {
}
