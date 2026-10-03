package ai.suncode.message.remote;

import java.util.List;

public record ProjectsData(List<ProjectDto> items, String encPayload) {
}
