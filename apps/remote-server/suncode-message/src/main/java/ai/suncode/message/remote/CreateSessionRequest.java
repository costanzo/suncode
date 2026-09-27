package ai.suncode.message.remote;

public record CreateSessionRequest(String hostId, String projectId, String title, String firstMessage) {
}
