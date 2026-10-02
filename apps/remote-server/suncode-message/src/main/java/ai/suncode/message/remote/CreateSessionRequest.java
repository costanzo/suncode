package ai.suncode.message.remote;

public record CreateSessionRequest(String projectId, String title, String firstMessage, java.util.List<ImageInput> images) {
}
