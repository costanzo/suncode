import { useState } from "react";
import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { WorkspaceGuideState } from "../WorkspaceGuide.jsx";
import { ProviderTracePanel } from "../WorkspacePrimitives.jsx";

const providerTraceGuides = {
  noTurns: {
    tabs: {
      actions: [
        "Use Conversation to submit a message and create the first turn.",
        "Return here after the provider responds to inspect model exchanges.",
        "Keep this state as the baseline for an empty trace.",
      ],
      style: [
        "The empty trace centers a 22px activity icon with 12px title text.",
        "The drawer keeps a 36px toolbar even when the list has no rows.",
        "Empty copy uses 10px muted text with a 1.5 line-height.",
      ],
      logic: [
        "The session has no persisted provider turns yet.",
        "There are no calls, tool exchanges, or token metrics to select.",
        "The first completed or in-flight model call creates a trace entry.",
      ],
    },
  },
  turnCollapsed: {
    tabs: {
      actions: [
        "Select the turn row to focus its summary metadata.",
        "Expand the turn when you need to inspect its calls.",
        "Use the toolbar copy action for the visible trace snapshot.",
      ],
      style: [
        "Turn rows use compact 40px minimum heights with a 7px content gap.",
        "Collapsed calls preserve title, time, status, and tokens without nested rows.",
        "The trace list uses a 230px desktop column before the detail divider.",
      ],
      logic: [
        "A turn exists but its model calls are collapsed for scanning.",
        "Collapsing changes presentation only; no trace data is discarded.",
        "The selected turn remains the source for the detail pane.",
      ],
    },
  },
  turnExpanded: {
    tabs: {
      actions: [
        "Expand a turn to reveal its model calls.",
        "Select a call to inspect its request, response, usage, and timing.",
        "Move between calls without leaving the provider trace.",
      ],
      style: [
        "The hierarchy uses turn and call rows only; calls are leaf rows",
        "Call rows keep title, time, status, and token summary aligned in one row.",
        "Selecting a call opens its overview detail without a nested content tier.",
      ],
      logic: [
        "Each turn can contain multiple model calls.",
        "Each call is a leaf row whose messages, tools, request, and response live in the detail pane.",
        "Expansion state is local UI state and does not alter canonical messages.",
      ],
    },
  },
  contextCompaction: {
    tabs: {
      actions: [
        "Select the Context compaction call to inspect the recorded event.",
        "Use the context summary to confirm how many messages and tokens were retained.",
        "Compare the event with the following model call when investigating context behavior.",
      ],
      style: [
        "The compaction call uses a subtle steel-tinted row.",
        "Its detail pane keeps the same metrics grid while replacing model response text with the compaction result.",
        "The turn and call hierarchy keeps system evenets easy to locate without a content tier.",
      ],
      logic: [
        "Context compaction is recorded as a completed internal event within the turn trace.",
        "It summarizes earlier messages before the next provider request; it is not a tool call.",
        "Retained and dropped token/message counts are trace metadata and do not expose credentials.",
      ],
    },
  },
  expanded: {
    tabs: {
      actions: [
        "Select a call to inspect its request, response, and usage",
        "Review request and response identifiers without exposing credentials.",
        "Use timing and token metrics to understand provider behavior.",
      ],
      style: [
        "The detail pane uses 14px padding and 10px vertical gaps between metadata blocks.",
        "Payloads render in 9px monospace text inside inset surfaces with 6px corners.",
        "The detail header keeps title, status, and token metrics aligned in a compact row.",
      ],
      logic: [
        "The selected call is read-only and reflects the normalized provider trace.",
        "Provider identifiers are redacted or presented as safe metadata.",
        "Usage and timing belong to the selected model call, not the whole session.",
      ],
    },
  },
};

export function WorkspaceProviderTracePage() {
  const [openGuide, setOpenGuide] = useState(null);
  const states = [
    {
      id: "noTurns",
      title: "No turns",
      description: "The session has no provider calls to display yet.",
      side: "right",
      content: <ProviderTracePanel standalone state="no-turns" />,
    },
    {
      id: "turnCollapsed",
      title: "Turn collapsed",
      description: "Turn summaries are visible while nested calls stay compact.",
      side: "left",
      content: <ProviderTracePanel standalone state="turn-collapsed" />,
    },
    {
      id: "turnExpanded",
      title: "Turn expanded",
      description: "A turn reveals its individual model calls.",
      side: "right",
      content: <ProviderTracePanel standalone state="turn-expanded" />,
    },
    {
      id: "contextCompaction",
      title: "Context compaction",
      description:
        "The turn includes a recorded context compaction call before the next model exchange.",
      side: "left",
      content: <ProviderTracePanel standalone state="context-compaction" />,
    },
    {
      id: "expanded",
      title: "Exchange expanded",
      description: "A selected exchange exposes canonical payload and metrics.",
      side: "left",
      content: <ProviderTracePanel standalone state="expanded" />,
    },
  ];
  return (
    <>
      <PageHeader
        title="Provider trace"
        description="Model exchanges, tool activity, usage, timing, and redacted provider identifiers."
      />
      <Section id="provider-trace-panel" title="Model exchange detail">
        <div className="workspace-state-grid workspace-drawer-state-grid">
          {states.map((state) => (
            <WorkspaceGuideState
              key={state.id}
              title={state.title}
              description={state.description}
              guide={providerTraceGuides[state.id]}
              side={state.side}
              open={openGuide === state.id}
              onToggle={() => setOpenGuide(openGuide === state.id ? null : state.id)}
              onClose={() => setOpenGuide(null)}
            >
              {state.content}
            </WorkspaceGuideState>
          ))}
        </div>
      </Section>
    </>
  );
}
