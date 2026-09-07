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
  modelUnavailable: {
    tabs: {
      actions: [
        "Review the conversation history while the selected model remains unavailable.",
        "Open Settings and add the provider API key before starting another turn.",
        "Choose a configured model when one is available in the model menu.",
      ],
      style: [
        "The composer stays visible but uses a quiet warning strip to explain why sending is disabled.",
        "The textarea and send action use the disabled treatment without hiding the model selector.",
        "The warning copy is compact and anchored to the composer rather than the message timeline.",
      ],
      logic: [
        "The selected model does not have a stored provider API key.",
        "Existing messages remain readable, but new submissions are blocked until a configured model is selected.",
        "Model selection remains available so the user can switch models without leaving the conversation.",
      ],
    },
  },
  waiting: {
    tabs: {
      actions: [
        "Read assistant responses in chronological order.",
        "Hover or focus a turn marker to preview the submitted user message.",
        "Click the changes summary to open Source Control for the turn diff.",
      ],
      style: [
        "Assistant Markdown uses 14px type with a 1.6 line-height for comfortable reading.",
        "Turn markers use a quiet rule, compact identifier, and bounded hover preview.",
        "Historical tool calls stay in Tool activity instead of the conversation timeline.",
      ],
      logic: [
        "The session has content and is waiting for the next user submission.",
        "The conversation keeps assistant responses and turn boundaries while user bodies move into marker previews.",
        "Turn changes summarize added, deleted, and edited files and link to the diff.",
      ],
    },
  },
  updating: {
    tabs: {
      actions: [
        "Watch the live work indicator while the current turn is executing.",
        "Select the active tool row to open its Tool activity detail.",
        "Wait for the assistant response before starting another turn.",
      ],
      style: [
        "The running indicator uses three 5px dots with staggered .12s delays.",
        "Each dot bounces in sequence beneath the conversation while the session is updating.",
        "Only the most recent running tool appears as a compact actionable row.",
        "The composer remains visually present while send is disabled during the active state.",
      ],
      logic: [
        "The current session has content and an active turn is updating it.",
        "Historical tool activity is available in the dedicated bottom drawer.",
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
        "Submitted images remain associated with the hidden user message and provider context.",
      ],
      logic: [
        "Only models that advertise image input enable the attachment control; the illustrative specimen model is not a seeded runtime model.",
        "Pending attachments stay local to the composer until send.",
        "Sent attachments become part of the durable user message even though the timeline stays assistant-first.",
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
        "With the title and close affordance hidden, the drafting surface starts at the same 20px inset as the modal's horizontal edges.",
        "The live character count sits below the drafting field on the left, aligned with its content edge.",
      ],
      logic: [
        "The modal edits the same draft as the compact composer so closing it does not lose work.",
        "The expanded state is a conversation-surface behavior, not a separate page or workflow.",
        "Sending from the expanded composer uses the same submission path as the compact composer.",
      ],
    },
  },
  liveToolStream: {
    tabs: {
      actions: [
        "Open the running tool row while it is still executing.",
        "Follow the live output in the selected Tool activity detail.",
        "Return to the conversation without losing the selected call.",
      ],
      style: [
        "The running tool row keeps the timeline compact and shows a warm live status instead of a completed success tone.",
        "The Tool activity drawer owns the bounded monospace output viewport.",
        "Request, live output, and completion summary stay in one detail pane.",
      ],
      logic: [
        "Long-running process tools can surface incremental output before the assistant reply completes.",
        "The Tool activity detail is inspection-only and keeps the main conversation readable.",
        "Completed tools stay in the turn tree while the conversation removes their rows.",
      ],
    },
  },
  scrolledUp: {
    tabs: {
      actions: [
        "Scroll up to review earlier messages in a long conversation.",
        "Use the floating down-arrow control to return to the newest message.",
        "Continue reading from the bottom without manually dragging the scrollbar.",
      ],
      style: [
        "The return-to-bottom control is a compact circular button above the composer.",
        "It uses the accent color sparingly and stays subordinate to the conversation content.",
        "The control appears only while the conversation is scrolled away from its latest message.",
      ],
      logic: [
        "The conversation viewport tracks whether the user is near the latest message.",
        "New content follows the tail only while the user remains at the bottom.",
        "Activating the control scrolls to the newest message and hides the control.",
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
      id: "modelUnavailable",
      title: "Model unavailable",
      description: "The selected provider has no API key, so new messages cannot be sent.",
      side: "left",
      content: <ConversationPanel standalone state="model-unavailable" onViewChanges={viewChanges} />,
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
      id: "immersiveComposer",
      title: "Expanded composer",
      description: "A large drafting modal opens from the compact composer when requested.",
      side: "left",
      content: <ConversationPanel standalone state="immersive-composer" onViewChanges={viewChanges} />,
    },
    {
      id: "liveToolStream",
      title: "Live tool output",
      description: "Click the single running command to open its detail in Tool activity.",
      side: "right",
      content: <ConversationPanel standalone state="live-tool-stream" />,
    },
    {
      id: "scrolledUp",
      title: "Scrolled up",
      description: "Earlier messages are in view and the newest message is below the viewport.",
      side: "left",
      content: <ConversationPanel standalone state="scrolled-up" onViewChanges={viewChanges} />,
    },
  ];
  return (
    <>
      <PageHeader
        title="Conversation"
        description="Assistant responses, lightweight turn boundaries, the current tool, and the turn composer."
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
