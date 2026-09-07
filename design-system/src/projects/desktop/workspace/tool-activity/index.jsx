import { useState } from "react";
import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { WorkspaceGuideState } from "../WorkspaceGuide.jsx";
import { ToolActivityPanel } from "../WorkspacePrimitives.jsx";

const guides = {
  running: {
    tabs: {
      actions: ["Expand a turn to inspect its tool calls.", "Select the running call to follow output.", "Use the inline conversation call to return here."],
      style: ["Turn rows use a compact separator and a one-line user preview.", "Tool rows are single-line entries: unfinished calls use a pulsing dot, completed calls stay quiet, and failures use danger text.", "The detail pane keeps request, output, and result in a single reading column."],
      logic: ["The active turn is expanded and the active tool is selected by default.", "Live output is best-effort and remains visibly distinct from the terminal result.", "Selection is presentation state and does not change the canonical session."],
    },
  },
  completed: {
    tabs: {
      actions: ["Expand the completed turn to review every executed tool.", "Select a tool to inspect its request and terminal result.", "Return to Conversation when only the assistant response is needed."],
      style: ["Every tool row uses one compact line with no redundant Completed label or status dot.", "The detail pane keeps terminal results readable without live emphasis.", "Historical calls remain selectable after the turn ends."],
      logic: ["The turn and each of its tool calls are terminally completed.", "Completed tools no longer appear as inline conversation rows.", "Selection does not reopen or rerun an operation."],
    },
  },
  empty: {
    tabs: {
      actions: ["Start a turn from Conversation to create the first activity entry.", "Keep the empty drawer available from the workspace gutter.", "Use Provider trace for model exchanges when no tools ran."],
      style: ["The empty state centers one activity icon and concise copy.", "The toolbar remains stable so the panel does not jump when the first call arrives.", "No placeholder tree rows are shown."],
      logic: ["A session can exist without any tool calls.", "The panel is not an error state.", "The first admitted tool call creates the first turn row."],
    },
  },
};

export function WorkspaceToolActivityPage() {
  const [openGuide, setOpenGuide] = useState(null);
  const states = [
    { id: "empty", title: "No turns", description: "The selected session has not admitted a turn yet.", side: "right", content: <ToolActivityPanel standalone state="empty" /> },
    { id: "running", title: "Active turn", description: "One executing turn is expanded with its complete tool list.", side: "left", content: <ToolActivityPanel standalone state="running" /> },
    { id: "completed", title: "Completed turn", description: "The turn and every tool call have completed.", side: "right", content: <ToolActivityPanel standalone state="completed" /> },
  ];
  return <>
    <PageHeader title="Tool activity" description="Turn-scoped tool calls, live output, status, and inspectable operation details." />
    <Section id="tool-activity-panel" title="Turn and tool progress">
      <div className="workspace-state-grid workspace-drawer-state-grid">
        {states.map((state) => <WorkspaceGuideState key={state.id} title={state.title} description={state.description} guide={guides[state.id]} side={state.side} open={openGuide === state.id} onToggle={() => setOpenGuide(openGuide === state.id ? null : state.id)} onClose={() => setOpenGuide(null)}>{state.content}</WorkspaceGuideState>)}
      </div>
    </Section>
  </>;
}
