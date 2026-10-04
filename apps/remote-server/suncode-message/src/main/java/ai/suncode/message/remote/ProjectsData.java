package ai.suncode.message.remote;

import java.util.List;

public record ProjectsData(List<ProjectDto> items, String encPayload) {
    public static ProjectsData encrypted(String encPayload) {
        return new ProjectsData(null, encPayload);
    }
}
