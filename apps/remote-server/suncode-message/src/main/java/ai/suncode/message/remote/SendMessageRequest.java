package ai.suncode.message.remote;

public record SendMessageRequest(String text, java.util.List<ImageInput> images) {
}
