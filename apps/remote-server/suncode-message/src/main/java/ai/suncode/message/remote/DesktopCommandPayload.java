package ai.suncode.message.remote;

import com.fasterxml.jackson.annotation.JsonInclude;
import lombok.Getter;
import lombok.Setter;

import java.util.List;

/** Fields accepted by the Desktop command operations. Unused fields remain null. */
@Getter
@Setter
@JsonInclude(JsonInclude.Include.NON_NULL)
public class DesktopCommandPayload {
    private String projectId;
    private String cursor;
    private Integer limit;
    private String title;
    private String firstMessage;
    private String text;
    private List<ImageInput> images;
    private String approvalId;
    private String action;
    private Integer expectedRevision;
    private String questionId;
    private List<String> answers;

    public static DesktopCommandPayload listSessions(String projectId, String cursor, Integer limit) {
        DesktopCommandPayload payload = new DesktopCommandPayload();
        payload.setProjectId(projectId == null ? "" : projectId);
        payload.setCursor(cursor == null ? "" : cursor);
        payload.setLimit(limit == null ? 50 : limit);
        return payload;
    }

    public static DesktopCommandPayload createSession(CreateSessionRequest request) {
        DesktopCommandPayload payload = new DesktopCommandPayload();
        payload.setProjectId(request.projectId());
        payload.setTitle(request.title());
        payload.setFirstMessage(request.firstMessage());
        payload.setImages(request.images());
        return payload;
    }

    public static DesktopCommandPayload sendMessage(SendMessageRequest request) {
        DesktopCommandPayload payload = new DesktopCommandPayload();
        payload.setText(request.text());
        payload.setImages(request.images());
        return payload;
    }

    public static DesktopCommandPayload resolveApproval(String approvalId, ApprovalResolutionRequest request) {
        DesktopCommandPayload payload = new DesktopCommandPayload();
        payload.setApprovalId(approvalId);
        payload.setAction(request.action());
        payload.setExpectedRevision(request.expectedRevision());
        return payload;
    }

    public static DesktopCommandPayload replyQuestion(String questionId, QuestionReplyRequest request) {
        DesktopCommandPayload payload = new DesktopCommandPayload();
        payload.setQuestionId(questionId);
        payload.setAnswers(request.answers());
        payload.setExpectedRevision(request.expectedRevision());
        return payload;
    }
}
