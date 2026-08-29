import { useState } from "react";
import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { WorkspaceGuideState } from "../WorkspaceGuide.jsx";
import { ReviewPanel } from "../WorkspacePrimitives.jsx";

const reviewGuides = {
  idle: { tabs: {
    actions: ["Start a turn from Conversation when the session is ready.", "Use this state to confirm no process is currently consuming authority.", "Review the session status again after submitting a message."],
    style: ["The heading uses a 14px title and a muted 7px status point.", "The empty panel centers a 22px activity icon with 12px title text.", "The review column is 312px wide with 14px top and 12px side padding."],
    logic: ["No turn is active and no approval or question is waiting.", "The agent can accept a new turn without requiring recovery.", "The panel intentionally hides turn changes when there is no active process."],
  } },
  runningNoChanges: { tabs: {
    actions: ["Watch the process card while the agent inspects the project.", "Use TODO to track work before any file mutation occurs.", "Open the live changes summary when the first modification arrives."],
    style: ["The process and TODO cards use 11px internal padding, 7px gaps, and 6px corners.", "The active status point uses an accent color with a 3px aura and 1.4s pulse.", "No-change copy remains secondary and does not compete with the active heading."],
    logic: ["A turn is running but has not produced file changes yet.", "The TODO projection remains visible while the agent loop is active.", "The state can transition to running-with-changes or a terminal outcome."],
  } },
  running: { tabs: {
    actions: ["Track the active process, model, latest activity, and current TODO items.", "Expand CHANGES to inspect the files modified by the current turn.", "Use the checkpoint action when the turn exposes an undoable boundary."],
    style: ["Running uses the accent status point and a 1.4s pulse animation.", "TODO rows are 16px tall with a 13px marker column and 6px content gap.", "The live changes summary uses a 40px minimum row with compact monospace counts."],
    logic: ["The agent owns an active turn and is still producing work.", "Changes are summarized separately from the process card and update in place.", "A completed turn removes the live marker and keeps the summary actionable."],
  } },
  approval: { tabs: {
    actions: ["Read the operation scope before selecting Allow once, Deny, or Allow for session.", "Use Allow once for a single execution with narrow authority.", "Deny stops the pending continuation without running the operation."],
    style: ["Approval uses the warning token and a 1.6s pulsing status point.", "The approval card uses 11px padding, 9px gaps, and a warning-tinted surface.", "Primary and danger actions are full-width or grouped with equal 30px control height."],
    logic: ["The turn is suspended before an authority-requiring operation executes.", "Approval is durable and scoped; it does not grant unrelated operations.", "The agent resumes only after an explicit decision is recorded."],
  } },
  question: { tabs: {
    actions: ["Choose one radio option, including the custom answer path.", "Enter additional context when the provided options are insufficient.", "Submit the answer to resume the suspended turn or skip the question."],
    style: ["Waiting for answer uses a success-colored 7px point with a 1.8s pulse.", "Options are stacked vertically with a 6px gap and 10px internal padding.", "Radio controls stay left-aligned while descriptions use 10px secondary text."],
    logic: ["The agent needs user input before it can choose the next action.", "A custom answer is valid only when its text is non-empty.", "Submitting or skipping resolves the question and resumes turn orchestration."],
  } },
  failed: { tabs: {
    actions: ["Read the failure reason and turn identifier in the stopped card.", "Use Retry turn to submit a new attempt after correcting the issue.", "Inspect Source Control separately to review any changes made before failure."],
    style: ["Failure uses the danger token with a static status point and danger-tinted surface.", "The failure card uses 11px padding, 9px gaps, and a 1px danger border.", "Reason and turn metadata use 9px monospace labels with right-aligned values."],
    logic: ["The turn ended before completion and no further tool calls will run.", "A failure is terminal for this turn but does not delete the session.", "Retry starts a new turn; existing history and any safe checkpoint remain inspectable."],
  } },
};

export function WorkspaceReviewPage() {
  const [openGuide, setOpenGuide] = useState(null);
  const states = [
    { id: "idle", title: "Idle", description: "No turn is active and the agent is ready for a new instruction.", side: "right", content: <ReviewPanel standalone state="idle" /> },
    { id: "runningNoChanges", title: "Running · no changes", description: "The agent is working but has not changed a file yet.", side: "left", content: <ReviewPanel standalone state="running-no-changes" /> },
    { id: "running", title: "Running", description: "The active turn has process activity and file changes.", side: "right", content: <ReviewPanel standalone state="running" /> },
    { id: "approval", title: "Waiting for approval", description: "A sensitive operation is paused for an explicit decision.", side: "left", content: <ReviewPanel standalone state="approval" /> },
    { id: "question", title: "Waiting for answer", description: "The agent needs a choice or custom input to continue.", side: "right", content: <ReviewPanel standalone state="question" /> },
    { id: "failed", title: "Turn failed", description: "The current turn stopped before completion.", side: "left", content: <ReviewPanel standalone state="failed" /> },
  ];
  return <><PageHeader title="Review" description="Agent progress, current-turn todos, approval decisions, questions, checkpoints, and undo entry points." /><Section id="review-panel" title="Agent sidebar"><div className="workspace-state-grid">{states.map((state) => <WorkspaceGuideState key={state.id} title={state.title} description={state.description} guide={reviewGuides[state.id]} side={state.side} open={openGuide === state.id} onToggle={() => setOpenGuide(openGuide === state.id ? null : state.id)} onClose={() => setOpenGuide(null)}>{state.content}</WorkspaceGuideState>)}</div></Section></>;
}
