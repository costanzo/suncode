import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { ReviewPanel } from "../WorkspacePrimitives.jsx";

export function WorkspaceReviewPage() {
  return <><PageHeader title="Review" description="Agent progress, current-turn todos, approval decisions, questions, checkpoints, and undo entry points." status="Phase 1" tone="implemented" /><Section id="review-panel" title="Agent sidebar"><ReviewPanel standalone /></Section></>;
}
