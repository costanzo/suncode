import { PageHeader, Section } from "../../../shared/PagePrimitives.jsx";
import { WorkspaceWindow } from "./WorkspacePrimitives.jsx";
import { WindowSizeNote } from "../WindowSizeNote.jsx";

export function WorkspacePage() {
  return (
    <>
      <PageHeader
        title="Workspace"
        description="The active project window, composed from the Avalonia session, conversation, review, and observability surfaces."
      />
      <WindowSizeNote width="1440" height="900" minimumWidth="620" minimumHeight="620" />
      <Section id="workspace-window" title="Project workspace">
        <WorkspaceWindow />
      </Section>
    </>
  );
}
