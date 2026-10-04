package ai.suncode.message.remote;

import java.util.List;

/** Rust {@code ProjectsResult} returned by the {@code projects.list} Desktop command. */
public record DesktopProjectsResult(List<DesktopProjectRecord> projects, String encPayload) {
}
