package ai.suncode.message.remote;

import com.fasterxml.jackson.databind.PropertyNamingStrategies;
import com.fasterxml.jackson.databind.annotation.JsonNaming;

import java.util.List;

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
public record EventContextSummary(
        String objective,
        List<String> importantConstraints,
        List<String> completedWork,
        List<String> activeWork,
        List<String> blockers,
        String nextAction) {
}
