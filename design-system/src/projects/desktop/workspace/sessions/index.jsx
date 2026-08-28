import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { SessionPanel } from "../WorkspacePrimitives.jsx";

export function WorkspaceSessionsPage() {
  const noSessions = [];
  const sessionsWithoutPins = [
    { title: "Provider migration review", time: "Yesterday" },
    { title: "Desktop navigation polish", time: "Aug 26" },
  ];
  const sessionsWithPins = [
    { title: "Workspace information architecture", time: "2 min ago", pinned: true },
    { title: "Provider migration review", time: "Yesterday", pinned: true },
    { title: "Desktop navigation polish", time: "Aug 26" },
  ];
  return <><PageHeader title="Sessions" description="Session selection, recency, pinning, and session actions for the open project." /><Section id="sessions-panel" title="Session navigation"><div className="workspace-session-state-grid"><div className="workspace-session-state"><h3>No sessions</h3><p>The project has no saved conversations yet.</p><SessionPanel standalone initialSessions={noSessions} /></div><div className="workspace-session-state"><h3>Sessions without pins</h3><p>Recent sessions are available, but none are pinned.</p><SessionPanel standalone initialSessions={sessionsWithoutPins} /></div><div className="workspace-session-state"><h3>Sessions with pins</h3><p>Pinned sessions stay at the top of the list for quick return.</p><SessionPanel standalone initialSessions={sessionsWithPins} /></div></div></Section></>;
}
