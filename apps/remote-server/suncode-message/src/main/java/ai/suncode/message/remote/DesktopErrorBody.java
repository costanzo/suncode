package ai.suncode.message.remote;

/** Failure body Desktop posts to {@code /v1/desktop/responses}; success bodies are the bare payload. */
public record DesktopErrorBody(Integer code, String message) {
}
