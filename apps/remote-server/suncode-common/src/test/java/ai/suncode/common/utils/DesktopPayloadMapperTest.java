package ai.suncode.common.utils;

import ai.suncode.message.remote.ConnectionState;
import ai.suncode.message.remote.HostDto;
import ai.suncode.message.remote.ProjectsData;
import ai.suncode.message.remote.SessionPageData;
import ai.suncode.message.remote.SessionSummaryDto;
import ai.suncode.message.remote.SyncData;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

class DesktopPayloadMapperTest {
    @Test
    void mapsRustProjectsResultWithoutAbsoluteRoots() {
        String body = """
                {"projects":[
                  {"projectId":"p-1","userId":"u","canonicalRoot":"/Users/me/app","displayName":"App",
                   "createdAt":"t","updatedAt":"t","lastOpenedAt":"t","archivedAt":null},
                  {"projectId":"p-2","canonicalRoot":"/old","displayName":"Old","archivedAt":"t"}]}
                """;
        ProjectsData data = DesktopPayloadMapper.projects(body);
        assertEquals(1, data.items().size());
        assertEquals("p-1", data.items().get(0).id());
        assertEquals("App", data.items().get(0).displayName());
        assertNull(data.items().get(0).relativeRoot());
        assertFalse(MarshallingUtils.toJson(data).contains("/Users/me"));
    }

    @Test
    void mapsRustSessionsResultToMobileSummaries() {
        String body = """
                {"project_id":"p-1","sessionStates":{"s-1":"approval","s-2":"running"},
                 "sessions":[
                   {"sessionId":"s-1","projectId":"p-1","title":"Fix","kind":"primary","status":"active",
                    "updatedAt":"2026-10-01T00:00:00Z","lastActivityAt":"2026-10-01T00:00:00Z"},
                   {"sessionId":"s-2","projectId":"p-1","title":null,"kind":"primary","status":"active",
                    "updatedAt":"2026-10-02T00:00:00Z","lastActivityAt":"2026-10-02T00:00:00Z"},
                   {"sessionId":"child","projectId":"p-1","kind":"child","updatedAt":"2026-10-03T00:00:00Z"}]}
                """;
        SessionPageData page = DesktopPayloadMapper.sessions(body);
        assertEquals(2, page.items().size());
        assertFalse(page.hasMore());
        SessionSummaryDto newest = page.items().get(0);
        assertEquals("s-2", newest.id());
        assertEquals("running", newest.state());
        assertEquals("New session", newest.title());
        SessionSummaryDto waiting = page.items().get(1);
        assertEquals("waiting_for_approval", waiting.state());
        assertTrue(waiting.pendingApproval());
        assertEquals("p-1", waiting.project().id());
    }

    @Test
    void forwardsEncryptedBodiesUntouched() {
        String body = "{\"encPayload\":\"e2e-v1:cipher\"}";
        assertEquals("e2e-v1:cipher", DesktopPayloadMapper.projects(body).encPayload());
        assertNull(DesktopPayloadMapper.projects(body).items());
        assertEquals("e2e-v1:cipher", DesktopPayloadMapper.sessions(body).encPayload());
        assertEquals("e2e-v1:cipher", DesktopPayloadMapper.createdSession(body).getEncPayload());
    }

    @Test
    void detectsOnlyExactDesktopErrorBodies() {
        assertEquals("projectId is required",
                DesktopPayloadMapper.error("{\"code\":50400,\"message\":\"projectId is required\"}").orElseThrow().message());
        assertTrue(DesktopPayloadMapper.error("{\"sessionId\":\"s-1\"}").isEmpty());
        assertTrue(DesktopPayloadMapper.error("{\"turn_id\":\"t\",\"status\":\"cancelled\"}").isEmpty());
        assertTrue(DesktopPayloadMapper.error("{\"encPayload\":\"x\"}").isEmpty());
    }

    @Test
    void readsCreatedSessionIdFromRustSessionRecord() {
        assertEquals("s-9", DesktopPayloadMapper.createdSession("{\"sessionId\":\"s-9\",\"kind\":\"primary\"}").getSessionId());
    }

    @Test
    void mapsDesktopSnapshotToMobileSyncData() {
        String body = """
                {"projects":[{"projectId":"p-1","displayName":"App","canonicalRoot":"/Users/me/app"}],
                 "sessions":[
                   {"sessionId":"s-1","projectId":"p-1","title":"Fix","kind":"primary",
                    "updatedAt":"2026-10-01T00:00:00Z","lastActivityAt":"2026-10-01T00:00:00Z"},
                   {"sessionId":"s-2","projectId":"p-1","kind":"primary","archivedAt":"t"}],
                 "sessionStates":{"s-1":"question"}}
                """;
        HostDto host = new HostDto("host-1", "Dev PC", ConnectionState.CONNECTED, null, "0.1.0");
        SyncData sync = DesktopPayloadMapper.sync(body, host);
        assertTrue(sync.resetRequired());
        assertFalse(sync.hasMore());
        assertEquals(sync.cursor(), DesktopPayloadMapper.sync(body, host).cursor());
        assertEquals("host-1", sync.host().id());
        assertEquals(1, sync.sessions().size());
        SessionSummaryDto session = sync.sessions().get(0);
        assertEquals("waiting_for_answer", session.state());
        assertTrue(session.pendingQuestion());
        assertEquals("App", session.project().displayName());
        assertFalse(MarshallingUtils.toJson(sync).contains("/Users/me"));

        SyncData encrypted = DesktopPayloadMapper.sync("{\"encPayload\":\"e2e-v1:cipher\"}", host);
        assertEquals("e2e-v1:cipher", encrypted.encPayload());
        assertNull(encrypted.host());
    }
}
