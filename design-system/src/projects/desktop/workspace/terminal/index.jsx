import { useState } from "react";

import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { WorkspaceGuideState } from "../WorkspaceGuide.jsx";
import { TerminalPanel } from "../panels/terminal/index.js";

const guides = {
  running: {
    tabs: {
      actions: ["Enter a command in the focused prompt.", "Use the tab and restart controls from the toolbar."],
      style: ["The drawer uses a 36px header and a dense monospace viewport.", "Output and the active prompt share one reading column."],
      logic: ["The shell belongs to the desktop client and is independent from Agent sessions.", "The project root is the default working directory."],
    },
  },
  empty: {
    tabs: {
      actions: ["Open a project to create the first local shell."],
      style: ["The empty state keeps the drawer chrome stable and places one action at the center."],
      logic: ["A terminal cannot start without an opened project."],
    },
  },
  exited: {
    tabs: {
      actions: ["Restart the shell from the inline action."],
      style: ["Exit status stays in the terminal surface instead of becoming a global alert."],
      logic: ["The shell ended normally and can be started again without reopening the project."],
    },
  },
  error: {
    tabs: {
      actions: ["Restart after correcting the local shell configuration."],
      style: ["The failure uses the danger tone and keeps the recovery action adjacent."],
      logic: ["The process failed before a terminal session was established."],
    },
  },
};

export function WorkspaceTerminalPage() {
  const [openGuide, setOpenGuide] = useState(null);
  const states = [
    { id: "running", title: "Shell running", side: "left", content: <TerminalPanel standalone /> },
    { id: "empty", title: "No project", side: "right", content: <TerminalPanel standalone state="empty" /> },
    { id: "exited", title: "Shell exited", side: "left", content: <TerminalPanel standalone state="exited" /> },
    { id: "error", title: "Startup error", side: "right", content: <TerminalPanel standalone state="error" /> },
  ];

  return (
    <>
      <PageHeader title="Terminal" description="A local shell in the Workspace bottom drawer." />
      <Section id="terminal-panel" title="Local shell">
        <div className="workspace-state-grid workspace-drawer-state-grid">
          {states.map((item) => (
            <WorkspaceGuideState
              key={item.id}
              title={item.title}
              description=""
              guide={guides[item.id]}
              side={item.side}
              open={openGuide === item.id}
              onToggle={() => setOpenGuide(openGuide === item.id ? null : item.id)}
              onClose={() => setOpenGuide(null)}
            >
              {item.content}
            </WorkspaceGuideState>
          ))}
        </div>
      </Section>
    </>
  );
}
