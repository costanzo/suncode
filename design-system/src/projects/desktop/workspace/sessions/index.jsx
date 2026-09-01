import { useState } from "react";
import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { WorkspaceGuideState } from "../WorkspaceGuide.jsx";
import { SessionPanel } from "../WorkspacePrimitives.jsx";

const sessionGuides = {
  empty: {
    title: "No sessions",
    side: "right",
    tabs: {
      actions: [
        "Select New session to start a conversation.",
        "Enter a clear name in the modal, then confirm creation.",
        "Use the session list as the return point for future turns.",
      ],
      style: [
        "Panel header: 12px UI font at 600 weight, with 14px top, 16px side, and 10px bottom margins.",
        "Empty state: centered 22px icon, 12px title, 10px supporting text, and 44px top / 32px bottom padding.",
        "The panel uses a 6px radius and 1px graphite border; no status dot appears because there is no agent process.",
      ],
      logic: [
        "The project has no saved conversations.",
        "Creating a session inserts it at the top of the list and selects it.",
        "The empty state disappears as soon as the first session exists.",
      ],
    },
  },
  withoutPins: {
    title: "Sessions without pins",
    side: "right",
    tabs: {
      actions: [
        "Click a session to make it active.",
        "Open the more menu to rename, pin, or archive it.",
        "Pin sessions that need to stay visible at the top.",
      ],
      style: [
        "Rows are 48px high with a 4px list gap; titles are 12px at weight 500 and recency is 11px muted text.",
        "The list has 16px horizontal padding; the row content uses 4px top/bottom and 10px right padding.",
        "The right side reserves 16px for status and 28px for overflow actions; idle sessions keep that slot quiet.",
      ],
      logic: [
        "Recent sessions are available but none are marked as pinned.",
        "Pinning changes only the ordering metadata, not the conversation content.",
        "Archiving removes a session from this list after confirmation.",
      ],
    },
  },
  withPins: {
    title: "Sessions with pins",
    side: "left",
    tabs: {
      actions: [
        "Click a pinned session to return to it.",
        "Use the more menu to unpin, rename, or archive.",
        "Keep the pin for sessions that are revisited frequently.",
      ],
      style: [
        "Pinned rows use a 12px accent pin icon; the title remains 12px/500 and the recency line remains 11px.",
        "The selected row uses the active surface with a 1px strong border and keeps the 48px row height.",
        "Pin, status, and overflow controls occupy separate fixed columns: 18px, 16px, and 28px.",
      ],
      logic: [
        "Pinned sessions remain in the same list and are identified by metadata.",
        "Unpinning removes the pin cue while preserving recency.",
        "The list supports mixed pinned and unpinned sessions.",
      ],
    },
  },
  statuses: {
    title: "Agent status states",
    side: "left",
    tabs: {
      actions: [
        "Select a session to inspect its conversation and Review panel.",
        "Use the status dot as a quick signal before opening the session.",
        "Open Review for the full process, approval, question, or failure details.",
      ],
      style: [
        "Status dots are 7px circles with a 3px outer aura; running pulses at 1.4s, approval at 1.6s, and answer-needed at 1.8s.",
        "Accent, warning, success, and danger tokens distinguish running, approval, answer-needed, and failed states; idle has no visible pulse.",
        "The indicator sits in a fixed 16px column between session content and the 28px overflow action column.",
      ],
      logic: [
        "Idle means the session is ready but no turn is executing.",
        "Running covers an active turn, including the no-changes-yet variant.",
        "Approval, answer-needed, and failed map directly to Review panel states.",
      ],
    },
  },
  archiveConfirmation: {
    title: "Archive confirmation",
    side: "right",
    tabs: {
      actions: [
        "Review the exact session name before confirming the archive action.",
        "Choose Cancel, press Escape, or close the dialog to leave the session unchanged.",
        "Choose Archive session to remove it from the active list.",
      ],
      style: [
        "The shared confirmation dialog uses the raised graphite modal surface, strong border, and 14px radius.",
        "The affected session appears in an inset target block between the consequence copy and actions.",
        "Cancel receives initial keyboard focus; the explicit Archive session action uses the danger treatment.",
      ],
      logic: [
        "Opening Archive from a session menu never changes data immediately.",
        "Backdrop, close, Escape, and Cancel all dismiss the dialog without archiving.",
        "Only the explicit Archive session action removes the session from the active list.",
      ],
    },
  },
};

export function WorkspaceSessionsPage() {
  const [openGuide, setOpenGuide] = useState(null);
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
  const sessionsByAgentStatus = [
    { title: "Idle session", time: "Just now", status: "idle" },
    { title: "Workspace indexing", time: "1 min ago", status: "running" },
    { title: "Production build approval", time: "4 min ago", status: "approval" },
    { title: "Layout decision needed", time: "12 min ago", status: "question" },
    { title: "Provider request failed", time: "Yesterday", status: "failed" },
  ];
  const states = [
    {
      id: "empty",
      title: "No sessions",
      description: "The project has no saved conversations yet.",
      sessions: noSessions,
    },
    {
      id: "withoutPins",
      title: "Sessions without pins",
      description: "Recent sessions are available, but none are pinned.",
      sessions: sessionsWithoutPins,
    },
    {
      id: "withPins",
      title: "Sessions with pins",
      description: "Pinned sessions stay at the top of the list for quick return.",
      sessions: sessionsWithPins,
    },
    {
      id: "statuses",
      title: "Agent status states",
      description: "Five sessions mapped to the Review panel status variants.",
      sessions: sessionsByAgentStatus,
    },
    {
      id: "archiveConfirmation",
      title: "Archive confirmation",
      description: "A shared confirmation dialog protects the consequential session action.",
      sessions: sessionsWithoutPins,
      initialArchiveConfirmation: true,
    },
  ];
  return (
    <>
      <PageHeader
        title="Sessions"
        description="Session selection, recency, pinning, and session actions for the open project."
      />
      <Section id="sessions-panel" title="Session navigation">
        <div className="workspace-session-state-grid">
          {states.map((state) => {
            const guide = sessionGuides[state.id];
            const guideOpen = openGuide === state.id;
            return (
              <WorkspaceGuideState
                key={state.id}
                className="workspace-session-state"
                title={state.title}
                description={state.description}
                guide={guide}
                side={guide.side}
                open={guideOpen}
                onToggle={() => setOpenGuide(guideOpen ? null : state.id)}
                onClose={() => setOpenGuide(null)}
              >
                <SessionPanel
                  standalone
                  initialSessions={state.sessions}
                  initialArchiveConfirmation={state.initialArchiveConfirmation}
                />
              </WorkspaceGuideState>
            );
          })}
        </div>
      </Section>
    </>
  );
}
