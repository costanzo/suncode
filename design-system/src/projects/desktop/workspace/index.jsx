import { PageHeader, Section } from "../../../shared/PagePrimitives.jsx";
import { WorkspaceWindow } from "./WorkspacePrimitives.jsx";

export function WorkspacePage() {
  return <><PageHeader title="Workspace" description="The active project window, composed from the Avalonia session, conversation, review, and observability surfaces." status="Phase 1" tone="implemented" /><Section id="workspace-window" title="Project workspace"><WorkspaceWindow /></Section></>;
}
