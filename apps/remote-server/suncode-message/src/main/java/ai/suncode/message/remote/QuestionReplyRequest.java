package ai.suncode.message.remote;

import java.util.List;

public record QuestionReplyRequest(List<String> answers, int expectedRevision) {
}
