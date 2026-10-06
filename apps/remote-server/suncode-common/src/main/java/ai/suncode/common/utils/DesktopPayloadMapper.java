package ai.suncode.common.utils;

import ai.suncode.message.remote.CreateSessionResponse;
import ai.suncode.message.remote.DesktopErrorBody;
import ai.suncode.message.remote.DesktopProjectRecord;
import ai.suncode.message.remote.DesktopProjectsResult;
import ai.suncode.message.remote.DesktopSessionRecord;
import ai.suncode.message.remote.DesktopSessionsResult;
import ai.suncode.message.remote.DesktopSnapshot;
import ai.suncode.message.remote.HostDto;
import ai.suncode.message.remote.ProjectDto;
import ai.suncode.message.remote.ProjectRefDto;
import ai.suncode.message.remote.ProjectsData;
import ai.suncode.message.remote.SessionPageData;
import ai.suncode.message.remote.SessionSummaryDto;
import ai.suncode.message.remote.SyncData;

import java.nio.charset.StandardCharsets;
import java.util.Comparator;
import java.util.HexFormat;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.stream.Collectors;

/**
 * Translates the bare Rust SDK results Desktop posts to {@code /v1/desktop/responses} into the Mobile
 * contract DTOs. Encrypted bodies ({@code encPayload}) are forwarded untouched; the Server never decrypts them.
 */
public final class DesktopPayloadMapper {
    private static final String PRIMARY_KIND = "primary";
    private static final String DEFAULT_TITLE = "New session";

    private DesktopPayloadMapper() {
    }

    /** Returns the Desktop failure when the body is exactly {@code {code, message}}, otherwise empty. */
    public static Optional<DesktopErrorBody> error(String body) {
        if (body == null || body.isBlank()) {
            return Optional.empty();
        }
        Map<?, ?> fields;
        try {
            fields = MarshallingUtils.fromJson(body, Map.class);
        } catch (RuntimeException ignored) {
            return Optional.empty();
        }
        if (fields == null || fields.size() != 2
                || !(fields.get("code") instanceof Number code)
                || !(fields.get("message") instanceof String message)) {
            return Optional.empty();
        }
        return Optional.of(new DesktopErrorBody(code.intValue(), message));
    }

    public static ProjectsData projects(String body) {
        DesktopProjectsResult result = MarshallingUtils.fromJson(body, DesktopProjectsResult.class);
        if (result.encPayload() != null) {
            return ProjectsData.encrypted(result.encPayload());
        }
        List<ProjectDto> items = Optional.ofNullable(result.projects()).orElse(List.of()).stream()
                .filter(Objects::nonNull)
                .filter(project -> project.archivedAt() == null)
                // relativeRoot stays null: Desktop only knows the absolute canonical root.
                .map(project -> new ProjectDto(project.projectId(), displayName(project), null, 0))
                .toList();
        return new ProjectsData(items, null);
    }

    public static SessionPageData sessions(String body) {
        DesktopSessionsResult result = MarshallingUtils.fromJson(body, DesktopSessionsResult.class);
        if (result.encPayload() != null) {
            return SessionPageData.encrypted(result.encPayload());
        }
        List<SessionSummaryDto> items = summaries(result.sessions(), result.sessionStates(), Map.of(), result.projectId());
        // The Rust SDK returns the full project list in one page.
        return new SessionPageData(items, null, false, null);
    }

    /**
     * Maps the stored Desktop snapshot to Mobile {@code SyncData}. Each snapshot is a complete projection, so it
     * is always a single reset page; the cursor is a content hash so an unchanged snapshot keeps the same cursor.
     */
    public static SyncData sync(String body, HostDto host) {
        DesktopSnapshot snapshot = MarshallingUtils.fromJson(body, DesktopSnapshot.class);
        if (snapshot.encPayload() != null) {
            return SyncData.encrypted(snapshot.encPayload());
        }
        Map<String, String> projectNames = Optional.ofNullable(snapshot.projects()).orElse(List.of()).stream()
                .filter(Objects::nonNull)
                .filter(project -> project.projectId() != null)
                .collect(Collectors.toMap(DesktopProjectRecord::projectId, DesktopPayloadMapper::displayName,
                        (first, ignored) -> first));
        List<SessionSummaryDto> sessions = summaries(snapshot.sessions(), snapshot.sessionStates(), projectNames, null);
        return new SyncData(cursor(body), true, host, sessions, false, null);
    }

    /** Maps Rust session UI state onto the Mobile {@code SessionState} enum. */
    public static String mobileState(String desktopState) {
        if (desktopState == null) {
            return "idle";
        }
        return switch (desktopState) {
            case "approval" -> "waiting_for_approval";
            case "question" -> "waiting_for_answer";
            case "running", "failed", "idle" -> desktopState;
            default -> "idle";
        };
    }

    private static List<SessionSummaryDto> summaries(List<DesktopSessionRecord> sessions, Map<String, String> states,
                                                     Map<String, String> projectNames, String fallbackProjectId) {
        Map<String, String> stateById = Optional.ofNullable(states).orElse(Map.of());
        return Optional.ofNullable(sessions).orElse(List.of()).stream()
                .filter(Objects::nonNull)
                .filter(session -> session.archivedAt() == null)
                .filter(session -> session.kind() == null || PRIMARY_KIND.equals(session.kind()))
                .sorted(Comparator.comparing(DesktopPayloadMapper::activityAt,
                        Comparator.nullsLast(Comparator.reverseOrder())))
                .map(session -> summary(session, fallbackProjectId, projectNames, stateById.get(session.sessionId())))
                .toList();
    }

    private static SessionSummaryDto summary(DesktopSessionRecord session, String fallbackProjectId,
                                             Map<String, String> projectNames, String state) {
        String projectId = session.projectId() != null ? session.projectId() : fallbackProjectId;
        String mobileState = mobileState(state);
        return new SessionSummaryDto(
                session.sessionId(),
                session.title() == null || session.title().isBlank() ? DEFAULT_TITLE : session.title(),
                PRIMARY_KIND,
                // sessions.list does not carry the project name; Mobile resolves it from projects.list.
                new ProjectRefDto(projectId, projectNames.getOrDefault(projectId, projectId)),
                mobileState,
                activityAt(session),
                "",
                "waiting_for_approval".equals(mobileState),
                "waiting_for_answer".equals(mobileState));
    }

    private static String activityAt(DesktopSessionRecord session) {
        return session.lastActivityAt() != null ? session.lastActivityAt() : session.updatedAt();
    }

    private static String displayName(DesktopProjectRecord project) {
        return project.displayName() == null || project.displayName().isBlank()
                ? project.projectId()
                : project.displayName();
    }

    private static String cursor(String body) {
        try {
            byte[] digest = java.security.MessageDigest.getInstance("SHA-256")
                    .digest(body.getBytes(StandardCharsets.UTF_8));
            return "sync-" + HexFormat.of().formatHex(digest, 0, 12);
        } catch (java.security.NoSuchAlgorithmException error) {
            throw new IllegalStateException("SHA-256 is unavailable", error);
        }
    }
}
