import { useState } from "react";
import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { WorkspaceGuideState } from "../WorkspaceGuide.jsx";
import { ConversationPanel, sampleConversationAttachments } from "../WorkspacePrimitives.jsx";

const conversationGuides = {
  noSession: { tabs: {
    actions: ["Choose a session from Sessions or create one with the plus action.", "Once selected, the composer becomes available for a new turn.", "Use this state to understand why no messages are shown yet."],
    style: ["The empty surface centers a 24px workspace icon with a 12px title.", "Supporting text uses 10px muted type and a 1.5 line-height.", "The conversation surface keeps 14px vertical and 24px horizontal padding."],
    logic: ["No session is selected, so there is no conversation history to render.", "The composer and Review context remain unavailable until a session exists.", "Selecting a session transitions to its current message state."],
  } },
  newSession: { tabs: {
    actions: ["Type an instruction in the composer to begin the first turn.", "Attach images before sending when visual context is needed.", "Use model and reasoning controls before submitting the turn."],
    style: ["The new-session state uses the same centered empty treatment as no-session.", "The composer is anchored at the bottom with a 16px radius and 1px strong border.", "Composer controls use 24px send and attachment buttons for compact reachability."],
    logic: ["A session exists but has no messages yet.", "The first submitted message creates the initial user turn.", "Attachments are held in the composer until the message is sent."],
  } },
  waiting: { tabs: {
    actions: ["Read the user and assistant messages in chronological order.", "Expand a tool operation to inspect its request and result modal.", "Click the changes summary to open Source Control for the turn diff."],
    style: ["User messages use a raised surface, 1px border, and 14px corner radius.", "Message text is 12px with a 1.62 line-height for comfortable reading.", "Tool rows are compact 34px controls with ellipsized operation names."],
    logic: ["The session has content and is waiting for the next user submission.", "Completed tool calls remain visible as auditable turn activity.", "Turn changes summarize added, deleted, and edited files and link to the diff."],
  } },
  updating: { tabs: {
    actions: ["Watch the live work indicator while the current turn is executing.", "Inspect tool calls as they complete without leaving the conversation.", "Wait for the assistant response before starting another turn."],
    style: ["The running indicator uses three 5px dots with staggered .12s delays.", "The process area stays compact and uses muted monospace metadata.", "The composer remains visually present while send is disabled during the active state."],
    logic: ["The current session has content and an active turn is updating it.", "Live tool activity is shown in sequence and may add turn changes.", "The final assistant message arrives when the turn completes."],
  } },
  attachments: { tabs: {
    actions: ["Use the plus control to select image files, up to three at a time.", "Click a thumbnail to open the larger preview modal.", "Hover a thumbnail and use its close control to remove it before sending."],
    style: ["Attachment thumbnails are 96px by 64px with a 7px radius and 6px gap.", "Thumbnails use cover cropping and a strong border on hover.", "Sent images appear above the user message with a 10px bottom margin."],
    logic: ["Only image MIME types are accepted by the attachment input.", "Pending attachments stay local to the composer until send.", "Sent attachments become part of the user message and can be previewed again."],
  } },
  longTool: { tabs: {
    actions: ["Read the truncated operation title to identify the active tool.", "Open the operation modal for the complete request and result.", "Use the status text to distinguish a long call from a failed call."],
    style: ["Long operation names are clipped with ellipsis inside a max-width row.", "The row keeps status and chevron controls visible at the right edge.", "The operation modal provides scrollable monospace request and result blocks."],
    logic: ["A long tool call is still one operation in the current turn.", "The list preserves full data in the modal while keeping the timeline compact.", "Tool completion state is independent from the assistant message that follows."],
  } },
};

export function WorkspaceConversationPage() {
  const [openGuide, setOpenGuide] = useState(null);
  const viewChanges = () => { window.location.hash = "/projects/desktop/workspace/source-control"; };
  const states = [
    { id: "noSession", title: "No session selected", description: "Nothing is selected, so the conversation surface is empty.", side: "right", content: <ConversationPanel standalone state="no-session" /> },
    { id: "newSession", title: "New session", description: "A session is ready for its first message.", side: "left", content: <ConversationPanel standalone state="new-session" /> },
    { id: "waiting", title: "Waiting for input", description: "A completed turn is waiting for the next instruction.", side: "right", content: <ConversationPanel standalone state="content-waiting" onViewChanges={viewChanges} /> },
    { id: "updating", title: "Session updating", description: "The active turn is streaming work and tool activity.", side: "left", content: <ConversationPanel standalone state="content-updating" /> },
    { id: "attachments", title: "Two images attached", description: "Two image thumbnails are held in the composer before sending.", side: "right", content: <ConversationPanel standalone state="content-waiting" initialAttachments={sampleConversationAttachments} onViewChanges={viewChanges} /> },
    { id: "attachments", title: "Two images sent", description: "The user message includes two sent image thumbnails.", side: "left", content: <ConversationPanel standalone state="content-waiting" initialSentAttachments={sampleConversationAttachments} onViewChanges={viewChanges} /> },
    { id: "longTool", title: "Long tool call", description: "A long operation title is compacted while its details remain available.", side: "right", content: <ConversationPanel standalone state="long-tool-call" onViewChanges={viewChanges} /> },
  ];
  return <><PageHeader title="Conversation" description="User messages, agent work, tool activity, final responses, and the turn composer." /><Section id="conversation-panel" title="Active conversation"><div className="workspace-state-grid workspace-conversation-state-grid">{states.map((state, index) => <WorkspaceGuideState key={`${state.id}-${index}`} title={state.title} description={state.description} guide={conversationGuides[state.id]} side={state.side} open={openGuide === `${state.id}-${index}`} onToggle={() => setOpenGuide(openGuide === `${state.id}-${index}` ? null : `${state.id}-${index}`)} onClose={() => setOpenGuide(null)}>{state.content}</WorkspaceGuideState>)}</div></Section></>;
}
