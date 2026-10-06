package ai.suncode.message.remote;

import com.fasterxml.jackson.annotation.JsonInclude;
import lombok.Getter;
import lombok.Setter;

/** Internal payload for Desktop operations whose request is assembled by the Server. */
@Getter
@Setter
@JsonInclude(JsonInclude.Include.NON_NULL)
public class DesktopCommandPayload {
    private String projectId;
    private String cursor;
    private Integer limit;

    public static DesktopCommandPayload listSessions(String projectId, String cursor, Integer limit) {
        DesktopCommandPayload payload = new DesktopCommandPayload();
        payload.setProjectId(projectId == null ? "" : projectId);
        payload.setCursor(cursor == null ? "" : cursor);
        payload.setLimit(limit == null ? 50 : limit);
        return payload;
    }
}
