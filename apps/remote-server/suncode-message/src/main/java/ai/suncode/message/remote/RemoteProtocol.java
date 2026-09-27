package ai.suncode.message.remote;

import com.fasterxml.jackson.databind.JsonNode;

import java.time.Instant;
import java.util.List;

/** Wire DTOs for the Java relay. The relay treats payloads as opaque protocol data. */
public final class RemoteProtocol {
    private RemoteProtocol() {
    }

    public record PairingExchangeRequest(
            String pairingPayload,
            String deviceName,
            String devicePublicKey,
            String clientNonce) {
    }

    public record PairingExchangeData(
            String accessToken,
            String refreshToken,
            Instant accessTokenExpiresAt,
            HostDto host) {
    }

    public record RefreshTokenRequest(String refreshToken) {
    }

    public record TokenData(String accessToken, String refreshToken, Instant accessTokenExpiresAt) {
    }

    public record HostDto(
            String id,
            String displayName,
            String endpoint,
            String connectionState,
            int projectCount,
            int activeSessionCount,
            Instant lastSeenAt,
            String desktopVersion,
            String protocolVersion) {
    }

    public record ProjectDto(String id, String displayName, String relativeRoot, int activeSessionCount) {
    }

    public record ProjectsData(List<ProjectDto> items) {
    }

    public record SessionPageData(List<JsonNode> items, String nextCursor, boolean hasMore) {
    }

    public record SyncData(
            String cursor,
            boolean resetRequired,
            List<HostDto> hosts,
            List<JsonNode> sessions,
            List<String> removedSessionIds,
            boolean hasMore) {
    }

    public record CommandAcceptedData(String requestId, Instant acceptedAt, String sessionId) {
    }

    public record CreateSessionRequest(String hostId, String projectId, String title, String firstMessage) {
    }

    public record SendMessageRequest(String text, String clientMessageId) {
    }

    public record ApprovalResolutionRequest(String action, int expectedRevision) {
    }

    public record QuestionReplyRequest(List<String> answers, int expectedRevision) {
    }

    public record DesktopCommand(
            String requestId,
            String hostId,
            String sessionId,
            String command,
            JsonNode payload) {
    }

    public record DesktopResponse(
            String requestId,
            String hostId,
            String sessionId,
            boolean success,
            Integer code,
            String message,
            JsonNode payload) {
    }

    public record DesktopEvent(
            String hostId,
            String sessionId,
            String requestId,
            String eventType,
            Instant occurredAt,
            JsonNode payload) {
    }

    public record MobileEvent(
            String eventId,
            long sequence,
            String hostId,
            String sessionId,
            String requestId,
            String eventType,
            Instant occurredAt,
            JsonNode payload) {
    }

    public record SessionSnapshot(
            String eventId,
            String sessionId,
            long sequence,
            long sessionRevision,
            JsonNode snapshot) {
    }
}
