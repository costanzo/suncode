package ai.suncode.message.remote;

public record EventProviderError(String code, String message, Boolean retryable) {
}
