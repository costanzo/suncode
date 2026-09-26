import { useState } from "react";
import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { WorkspaceGuideState } from "../WorkspaceGuide.jsx";
import {
  childSessions,
  ChildSessionDetail,
  ChildSessionsPanel,
  ContentSwitcher,
  primarySessions,
} from "../panels/child-sessions/index.js";

const guide = {
  tabs: {
    actions: [
      "Select a delegated session from the right panel to open its read-only detail in the central region.",
      "Use Back to main session or the ContentSwitcher to return to the parent conversation.",
      "Resolve a child machine-operation approval inline with Allow once or Deny; the detail never exposes a composer or direct reply action.",
    ],
    style: [
      "The right panel reuses the 312px supporting-bay geometry and the same compact row rhythm as Sessions.",
      "The central detail is a reading surface with a 44px identity header, quiet metadata, and one chronological activity spine.",
      "Agent identity stays neutral; warning and danger colors are reserved for approval and failure states.",
    ],
    logic: [
      "Child sessions belong to one primary session and never appear in the left primary Sessions list.",
      "Selecting a child changes only the inspected content; the primary session remains the authority and conversation root.",
      "The ContentSwitcher treats child sessions as a third content kind and restores them by stable child-session identity.",
    ],
  },
};

function childContent(child) {
  return {
    id: `child-session:${child.id}`,
    kind: "child-session",
    title: child.title,
    detail: `${child.agentDisplayName} · ${child.state}`,
    child,
  };
}

export function WorkspaceChildSessionsPage() {
  const [selected, setSelected] = useState(childSessions[0]);
  const [guideOpen, setGuideOpen] = useState(false);
  const recent = childSessions.map(childContent);
  const parent =
    primarySessions.find((session) => session.id === selected.parentSessionId) ??
    primarySessions[0];
  const siblings = childSessions.filter(
    (child) => child.parentSessionId === selected.parentSessionId,
  );
  const returnToParent = () => {
    window.location.hash = "/projects/desktop/workspace";
  };

  return (
    <>
      <PageHeader
        title="Child sessions"
        description="Delegated agent work as linked, read-only sessions inside the primary workspace."
      />
      <Section
        id="child-session-composition"
        title="Inspection flow"
        description="The right panel selects a child session; the central region shows its complete read-only activity."
      >
        <WorkspaceGuideState
          title="Parent-linked child sessions"
          description="A child session remains observable without becoming a second user conversation."
          guide={guide}
          side="left"
          open={guideOpen}
          onToggle={() => setGuideOpen((open) => !open)}
          onClose={() => setGuideOpen(false)}
        >
          <div className="workspace-child-specimen">
            <div className="workspace-child-specimen-titlebar">
              <span />
              <ContentSwitcher
                currentItem={childContent(selected)}
                items={recent}
                onSelect={(item) => setSelected(item.child)}
              />
              <span />
            </div>
            <div className="workspace-child-specimen-body">
              <ChildSessionDetail child={selected} onBack={returnToParent} />
              <ChildSessionsPanel
                sessions={siblings}
                selectedChildId={selected.id}
                parentTitle={parent.title}
                onSelectChild={setSelected}
              />
            </div>
          </div>
        </WorkspaceGuideState>
      </Section>

      <Section
        id="child-session-states"
        title="Detail states"
        description="Running, completed, approval, and failed sessions keep the same read-only structure while state-specific feedback changes."
      >
        <div className="workspace-child-detail-state-grid">
          {childSessions.map((child) => (
            <div key={child.id}>
              <h3>
                {child.agentDisplayName} · {child.state}
              </h3>
              <ChildSessionDetail child={child} standalone onBack={returnToParent} />
            </div>
          ))}
        </div>
      </Section>

      <Section
        id="child-session-empty"
        title="Empty state"
        description="The panel explains where child sessions come from and does not offer a create action."
      >
        <ChildSessionsPanel sessions={[]} standalone />
      </Section>
    </>
  );
}
