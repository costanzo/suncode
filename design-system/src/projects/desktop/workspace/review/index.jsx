import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { ReviewPanel } from "../WorkspacePrimitives.jsx";

export function WorkspaceReviewPage() {
  return <><PageHeader title="Review" description="Agent progress, current-turn todos, approval decisions, questions, checkpoints, and undo entry points." /><Section id="review-panel" title="Agent sidebar"><div className="workspace-state-grid"><div><h3>Idle</h3><ReviewPanel standalone state="idle" /></div><div><h3>Running · no changes</h3><ReviewPanel standalone state="running-no-changes" /></div><div><h3>Running</h3><ReviewPanel standalone state="running" /></div><div><h3>Waiting for approval</h3><ReviewPanel standalone state="approval" /></div><div><h3>Waiting for answer</h3><ReviewPanel standalone state="question" /></div><div><h3>Turn failed</h3><ReviewPanel standalone state="failed" /></div></div></Section></>;
}
