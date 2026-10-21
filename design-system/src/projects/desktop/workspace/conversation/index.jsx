import { useState } from "react";
import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { WorkspaceGuideState } from "../WorkspaceGuide.jsx";
import { ConversationPanel, sampleConversationAttachments } from "../panels/conversation/index.js";

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
        "After a new session is created, keyboard focus moves to the available composer so the first instruction can be typed immediately.",
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
        "Read user and assistant messages in chronological order.",
        "Use the latest duration label to distinguish an active turn from a completed turn.",
        "Click the changes summary to open Source Control for the turn diff.",
      ],
      style: [
        "Assistant Markdown uses 14px type with a 1.6 line-height for comfortable reading.",
        "User rows use a right-aligned message surface while assistant rows stay quiet and readable.",
        "The active tool row carries a muted Working for duration label, while completed assistant output shows Worked for.",
        "Completed assistant output places the completion time immediately beside the copy action.",
        "Historical tool calls stay in Tool activity instead of the conversation timeline.",
      ],
      logic: [
        "The session has content and is waiting for the next user submission.",
        "The conversation keeps submitted user messages and assistant responses in chronological order without per-turn divider metadata.",
        "Turn changes summarize added, deleted, and edited files and link to the diff.",
        "The completion time is presented as a quiet local-time marker with an accessible exact timestamp tooltip.",
      ],
    },
  },
  intermediateAssistant: {
    tabs: {
      actions: [
        "Compare the quieter intermediate assistant update with the final response below it.",
        "Use the intermediate update as progress context rather than the turn's conclusion.",
        "Continue to the final assistant message for the completed summary and actions.",
      ],
      style: [
        "Intermediate assistant messages use 12px muted text with a tighter 1.5 line-height.",
        "They omit completion duration, copy, and change actions so the final response remains primary.",
        "The final assistant message keeps the full 14px hierarchy, duration, completion time, and copy action.",
      ],
      logic: [
        "A turn may emit assistant updates before its final response is available.",
        "Intermediate updates stay in chronological order and remain readable without competing with the final message.",
        "Only the final assistant message owns completion metadata and response actions.",
      ],
    },
  },
  updating: {
    tabs: {
      actions: [
        "Watch the live work indicator while the current turn is executing.",
        "Select the active tool row to open its Tool activity detail.",
        "Send a follow-up for after the current turn, then choose Steer now on that queued message to guide this turn instead.",
      ],
      style: [
        "The running indicator uses three 5px dots with staggered .12s delays.",
        "Each dot bounces in sequence beneath the conversation while the session is updating.",
        "Only the most recent running tool appears as a compact actionable row.",
        "The composer stays available; the primary action queues input while the turn runs.",
      ],
      logic: [
        "The current session has content and an active turn is updating it.",
        "Historical tool activity is available in the dedicated bottom drawer.",
        "The final assistant message arrives when the turn completes.",
      ],
    },
  },
  followupSteering: {
    tabs: {
      actions: [
        "Send normally (Enter) to add text and images to Up next as a follow-up for after this turn.",
        "Choose Steer now on the right of a specific queued message to include it in the next model request of the current turn.",
        "Use Take back and edit or Remove on a queued row; text and images move together.",
        "Collapse hides only the message list. The composer, draft images, and controls stay available.",
      ],
      style: [
        "The composer has one primary send action. Steering appears only on the right of queued rows.",
        "Each row shows Follow-up · After this turn or Steering · Next model request. A promoted row shows Steering queued and cannot be promoted twice.",
        "The bounded list reserves space above the composer and scrolls at 180px. The collapsed header retains count, mode summary, and pause state.",
      ],
      logic: [
        "Follow-ups wait for the active turn to finish, then start new turns in FIFO order. They are not injected into the current loop automatically.",
        "Steering changes only the selected queued message. The current provider request and tools continue; the next model request consumes steering within the same turn. Other follow-ups keep their order and the unsent draft is untouched.",
        "Keep promoted text and images visible until admission is confirmed. Admission failure retains that row and offers Retry steer without cancelling current work.",
        "A vision-capable model permits up to three images per message, including image-only input. Model capability is revalidated at admission.",
        "Taking back a row restores the entire draft. An existing draft moves into that queue position as a follow-up and pauses the queue for explicit review/resume.",
        "Only the separate Stop action cancels work. It pauses pending messages; steering itself never pauses other follow-ups.",
        "Review-only proposal: pending and failed steering are seeded fixtures, not a simulated provider boundary. Rust currently exposes text continuation within an active turn; separate after-turn follow-ups, images, and queue management need runtime work.",
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
  retrying: {
    tabs: {
      actions: ["Wait for the scheduled request retry or use the composer stop action."],
      style: [
        "A static 11px secondary-text status replaces Thinking and running dots; it wraps at narrow widths.",
      ],
      logic: [
        "Show the scheduled delay and retry number while waiting. Clear it when the next request starts or the turn ends. The delay is a schedule, not a live countdown.",
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
        "Draft thumbnails use the 48px conversation thumbnail token; queued references use compact 36px by 32px thumbnails.",
        "Thumbnails use cover cropping and a strong border on hover.",
        "Submitted images remain associated with the visible user message and provider context.",
      ],
      logic: [
        "Only models that advertise image input enable the attachment control; the illustrative specimen model is not a seeded runtime model.",
        "Pending attachments stay local to the composer until send.",
        "Sent attachments become part of the durable user message and remain visible in its timeline row.",
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
        "The first entry into a session positions the viewport at its latest message and initially loads 10 timeline messages; reaching the top loads the preceding page of 10 with a stable cursor while preserving the viewport anchor.",
        "A session restores its previous viewport and already loaded pages only while it is among the project's 10 most recently viewed sessions and has activity within the past 24 hours.",
        "Submitting a new turn re-enters follow mode so the sent user message and live output remain visible.",
        "If the user remains at the latest message, turn completion may follow the final response to the newest message.",
        "After the user scrolls away, later assistant, tool, and layout updates preserve the current viewport instead of following new content.",
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
      content: (
        <ConversationPanel standalone state="model-unavailable" onViewChanges={viewChanges} />
      ),
    },
    {
      id: "waiting",
      title: "Waiting for input",
      description: "A completed turn is waiting for the next instruction.",
      side: "right",
      content: <ConversationPanel standalone state="content-waiting" onViewChanges={viewChanges} />,
    },
    {
      id: "intermediateAssistant",
      title: "Intermediate assistant message",
      description:
        "A quieter in-turn update keeps progress context secondary to the final response.",
      side: "right",
      content: (
        <ConversationPanel standalone state="intermediate-assistant" onViewChanges={viewChanges} />
      ),
    },
    {
      id: "updating",
      title: "Session updating",
      description: "The active turn is streaming work and tool activity.",
      side: "left",
      content: <ConversationPanel standalone state="content-updating" />,
    },
    {
      id: "followupSteering",
      title: "Follow-up and steering",
      description:
        "Send a follow-up by default. Choose Steer now on a queued row to guide the current turn at its next model request.",
      side: "right",
      content: (
        <ConversationPanel
          standalone
          state="queued-input"
          initialAttachments={[sampleConversationAttachments[1]]}
          imageInputEnabled
        />
      ),
    },
    {
      id: "collapsedQueue",
      guideId: "followupSteering",
      title: "Follow-up and steering · queue collapsed",
      description:
        "Only the queued messages fold into their header. The composer, draft image, and sending controls remain unchanged.",
      side: "right",
      content: (
        <ConversationPanel
          standalone
          state="queued-input"
          initialAttachments={[sampleConversationAttachments[1]]}
          imageInputEnabled
          initialQueueCollapsed
        />
      ),
    },
    {
      id: "steeringPending",
      guideId: "followupSteering",
      title: "Steering · pending",
      description:
        "The selected message waits for the next model request. Current output and tools continue; other messages remain follow-ups.",
      side: "right",
      content: (
        <ConversationPanel
          standalone
          state="queued-input"
          initialQueuedSteering="pending"
          initialAttachments={[sampleConversationAttachments[1]]}
          imageInputEnabled
        />
      ),
    },
    {
      id: "steeringFailed",
      guideId: "followupSteering",
      title: "Steering · admission failed",
      description:
        "The selected message and its images remain in the list. Retry steering from that row while current work continues.",
      side: "right",
      content: (
        <ConversationPanel
          standalone
          state="queued-input"
          initialQueuedSteering="failed"
          initialAttachments={[sampleConversationAttachments[1]]}
          imageInputEnabled
        />
      ),
    },
    {
      id: "thinking",
      title: "Assistant thinking",
      description: "A dedicated thinking phase replaces the generic three-dot running marker.",
      side: "right",
      content: <ConversationPanel standalone state="content-thinking" />,
    },
    {
      id: "retrying",
      title: "Request retry scheduled",
      description: "A temporary provider failure pauses the active turn before another request.",
      side: "left",
      content: <ConversationPanel standalone state="content-retrying" />,
    },
    {
      id: "attachments",
      title: "Two images attached",
      description:
        "A specimen-only image-capable model shows two thumbnails before and after sending.",
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
      id: "attachmentsSent",
      guideId: "attachments",
      title: "Attachments sent",
      description: "A completed user message keeps its image attachments in conversation history.",
      side: "right",
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
      description: "A large drafting modal opens from the compact composer when requested.",
      side: "left",
      content: (
        <ConversationPanel standalone state="immersive-composer" onViewChanges={viewChanges} />
      ),
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
              guide={conversationGuides[state.guideId ?? state.id]}
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
