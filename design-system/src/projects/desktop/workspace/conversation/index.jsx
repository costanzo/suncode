import { useState } from "react";
import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { WorkspaceGuideState } from "../WorkspaceGuide.jsx";
import { ConversationPanel, sampleConversationAttachments } from "../WorkspacePrimitives.jsx";

const conversationGuides = {
  noSession: {
    tabs: {
      actions: [
        "Choose a session from Sessions or create one with the plus action.",
        "Once selected, the composer becomes available for a new turn.",
        "Use this state to understand why no messages are shown yet.",
      ],
      style: [
        "The empty surface centers a 24px workspace icon with a 12px title.",
        "Supporting text uses 10px muted type and a 1.5 line-height.",
        "The conversation surface keeps 14px vertical and 24px horizontal padding.",
      ],
      logic: [
        "No session is selected, so there is no conversation history to render.",
        "The composer and Review context remain unavailable until a session exists.",
        "Selecting a session transitions to its current message state.",
      ],
    },
  },
  newSession: {
    tabs: {
      actions: [
        "Type an instruction in the composer to begin the first turn.",
        "Attach images before sending when visual context is needed.",
        "Use model and reasoning controls before submitting the turn.",
      ],
      style: [
        "The new-session state uses the same centered empty treatment as no-session.",
        "The composer is anchored at the bottom with a 16px radius and 1px strong border.",
        "Composer controls use 24px send and attachment buttons for compact reachability.",
      ],
      logic: [
        "A session exists but has no messages yet.",
        "The first submitted message creates the initial user turn.",
        "Attachments are held in the composer until the message is sent.",
      ],
    },
  },
  waiting: {
    tabs: {
      actions: [
        "Read the user and assistant messages in chronological order.",
        "Expand a tool operation to inspect its request and result modal.",
        "Click the changes summary to open Source Control for the turn diff.",
      ],
      style: [
        "User messages use a raised surface, 1px border, and 14px corner radius.",
        "Assistant Markdown uses 14px type with a 1.6 line-height for comfortable reading.",
        "Tool rows are compact 34px controls with ellipsized operation names.",
      ],
      logic: [
        "The session has content and is waiting for the next user submission.",
        "Completed tool calls remain visible as auditable turn activity.",
        "Turn changes summarize added, deleted, and edited files and link to the diff.",
      ],
    },
  },
  updating: {
    tabs: {
      actions: [
        "Watch the live work indicator while the current turn is executing.",
        "Inspect tool calls as they complete without leaving the conversation.",
        "Wait for the assistant response before starting another turn.",
      ],
      style: [
        "The running indicator uses three 5px dots with staggered .12s delays.",
        "The process area stays compact and uses muted monospace metadata.",
        "The composer remains visually present while send is disabled during the active state.",
      ],
      logic: [
        "The current session has content and an active turn is updating it.",
        "Live tool activity is shown in sequence and may add turn changes.",
        "The final assistant message arrives when the turn completes.",
      ],
    },
  },
  thinking: {
    tabs: {
      actions: [
        "Use this state to review the assistant's thinking phase before tool execution or final output appears.",
        "Confirm that the status remains readable without relying on bouncing dots.",
        "Check that the animation still feels calm and legible in the conversation timeline.",
      ],
      style: [
        "The thinking indicator uses the literal word Thinking instead of the three-dot running marker.",
        "Letters reveal from left to right in a repeating loop, keeping motion directional but restrained.",
        "The indicator stays low-noise and uses the same graphite conversation hierarchy as the rest of the surface.",
      ],
      logic: [
        "Thinking is a dedicated conversation state, separate from generic active tool execution.",
        "While thinking is shown, the standard three-dot running indicator is intentionally absent.",
        "The surface can transition from thinking into tool activity or a final assistant response.",
      ],
    },
  },
  compacted: {
    tabs: {
      actions: [
        "Use the event marker to confirm that the conversation has been compacted.",
        "Continue reading the current turn normally after the summary is applied.",
        "Open Provider trace when the exact model-call sequence needs inspection.",
      ],
      style: [
        "The completed event uses a static 7px steel point with no pulse or outer aura.",
        "The row uses the same 8px vertical and 10px horizontal rhythm as active compaction.",
        "A muted secondary line explains the result without adding another card or tool row.",
      ],
      logic: [
        "A previous context.compacted event has been received for this session.",
        "Earlier messages were summarized and the resulting context is now being used.",
        "This marker is historical feedback; it does not mean the agent is currently running.",
      ],
    },
  },
  attachments: {
    tabs: {
      actions: [
        "Use the plus control to select image files, up to three at a time.",
        "Click a thumbnail to open the larger preview modal.",
        "Hover a thumbnail and use its close control to remove it before sending.",
      ],
      style: [
        "Attachment thumbnails are 96px by 64px with a 7px radius and 6px gap.",
        "Thumbnails use cover cropping and a strong border on hover.",
        "Sent images appear above the user message with a 10px bottom margin.",
      ],
      logic: [
        "Only models that advertise image input enable the attachment control; the illustrative specimen model is not a seeded runtime model.",
        "Pending attachments stay local to the composer until send.",
        "Sent attachments become part of the user message and can be previewed again.",
      ],
    },
  },
  immersiveComposer: {
    tabs: {
      actions: [
        "Use the expand control in the composer footer when the compact field feels too small.",
        "Draft a multi-paragraph prompt in the larger modal surface.",
        "Watch the character counter update live in the lower-right corner while you type.",
      ],
      style: [
        "The expanded composer keeps the graphite dialog language and a quieter title treatment than a destructive modal.",
        "The large textarea uses the same UI type as the compact composer, but grows into an immersive drafting surface.",
        "Character feedback stays attached to the drafting area instead of adding extra chrome to the compact composer.",
      ],
      logic: [
        "The modal edits the same draft as the compact composer so closing it does not lose work.",
        "The expanded state is a conversation-surface behavior, not a separate page or workflow.",
        "Sending from the expanded composer uses the same submission path as the compact composer.",
      ],
    },
  },
  longTool: {
    tabs: {
      actions: [
        "Read the truncated operation title to identify the active tool.",
        "Open the operation modal for the complete request and result.",
        "Use the status text to distinguish a long call from a failed call.",
      ],
      style: [
        "Long operation names are clipped with ellipsis inside a max-width row.",
        "The row keeps status and chevron controls visible at the right edge.",
        "The operation modal provides scrollable monospace request and result blocks.",
      ],
      logic: [
        "A long tool call is still one operation in the current turn.",
        "The list preserves full data in the modal while keeping the timeline compact.",
        "Tool completion state is independent from the assistant message that follows.",
      ],
    },
  },
  liveToolStream: {
    tabs: {
      actions: [
        "Open the running tool row to inspect the command while it is still executing.",
        "Read the live output region in the modal to follow long operations such as compile commands.",
        "Use the request and status details to understand what is still running without leaving the conversation.",
      ],
      style: [
        "The running tool row keeps the timeline compact and shows a warm live status instead of a completed success tone.",
        "The modal adds a dedicated output viewport with monospace lines and vertical scrolling.",
        "Request, live output, and completion summary stay in one raised dialog rather than splitting the user into multiple panes.",
      ],
      logic: [
        "Long-running process tools can surface incremental output before the assistant reply completes.",
        "The live-output modal is inspection-only and keeps the main conversation readable.",
        "Completed tools still use the same modal shell, but the live pane becomes a historical output record.",
      ],
    },
  },
};

export function WorkspaceConversationPage() {
  const [openGuide, setOpenGuide] = useState(null);
  const viewChanges = () => {
    window.location.hash = "/projects/desktop/workspace/source-control";
  };
  const states = [
    {
      id: "noSession",
      title: "No session selected",
      description: "Nothing is selected, so the conversation surface is empty.",
      side: "right",
      content: <ConversationPanel standalone state="no-session" />,
    },
    {
      id: "newSession",
      title: "New session",
      description: "A session is ready for its first message.",
      side: "left",
      content: <ConversationPanel standalone state="new-session" />,
    },
    {
      id: "waiting",
      title: "Waiting for input",
      description: "A completed turn is waiting for the next instruction.",
      side: "right",
      content: <ConversationPanel standalone state="content-waiting" onViewChanges={viewChanges} />,
    },
    {
      id: "updating",
      title: "Session updating",
      description: "The active turn is streaming work and tool activity.",
      side: "left",
      content: <ConversationPanel standalone state="content-updating" />,
    },
    {
      id: "thinking",
      title: "Assistant thinking",
      description: "A dedicated thinking phase replaces the generic three-dot running marker.",
      side: "right",
      content: <ConversationPanel standalone state="content-thinking" />,
    },
    {
      id: "compacted",
      title: "Context compacted",
      description: "A completed context compaction is recorded in the conversation timeline.",
      side: "left",
      content: <ConversationPanel standalone state="context-compacted" />,
    },
    {
      id: "attachments",
      title: "Two images attached",
      description: "A specimen-only image-capable model holds two thumbnails before sending.",
      side: "right",
      content: (
        <ConversationPanel
          standalone
          state="content-waiting"
          initialAttachments={sampleConversationAttachments}
          imageInputEnabled
          onViewChanges={viewChanges}
        />
      ),
    },
    {
      id: "attachments",
      title: "Two images sent",
      description: "The user message includes two sent image thumbnails.",
      side: "left",
      content: (
        <ConversationPanel
          standalone
          state="content-waiting"
          initialSentAttachments={sampleConversationAttachments}
          onViewChanges={viewChanges}
        />
      ),
    },
    {
      id: "immersiveComposer",
      title: "Expanded composer",
      description: "A large drafting modal opens from the compact composer for longer prompts.",
      side: "left",
      content: <ConversationPanel standalone state="immersive-composer" onViewChanges={viewChanges} />,
    },
    {
      id: "longTool",
      title: "Long tool call",
      description: "A long operation title is compacted while its details remain available.",
      side: "right",
      content: <ConversationPanel standalone state="long-tool-call" onViewChanges={viewChanges} />,
    },
    {
      id: "liveToolStream",
      title: "Live tool output",
      description: "A running command opens a modal with streaming output while the turn is still active.",
      side: "right",
      content: <ConversationPanel standalone state="live-tool-stream" />,
    },
  ];
  return (
    <>
      <PageHeader
        title="Conversation"
        description="User messages, agent work, tool activity, final responses, and the turn composer."
      />
      <Section id="conversation-panel" title="Active conversation">
        <div className="workspace-state-grid workspace-conversation-state-grid">
          {states.map((state, index) => (
            <WorkspaceGuideState
              key={`${state.id}-${index}`}
              title={state.title}
              description={state.description}
              guide={conversationGuides[state.id]}
              side={state.side}
              open={openGuide === `${state.id}-${index}`}
              onToggle={() =>
                setOpenGuide(openGuide === `${state.id}-${index}` ? null : `${state.id}-${index}`)
              }
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
