import { useState } from "react";
import { Button } from "../../../components/universal/button/index.js";
import { ConfirmationDialog } from "../../../components/universal/modal/index.js";
import { PageHeader, Section } from "../../../shared/PagePrimitives.jsx";
import { NativeWindowFrame } from "../../../platforms/desktop/components/titlebar/index.js";
import { WorkspaceGuideState } from "../workspace/WorkspaceGuide.jsx";
import { WindowSizeNote } from "../WindowSizeNote.jsx";

const dialogGuide = {
  tabs: {
    actions: [
      "Click the action button to open the confirmation dialog.",
      "Review the affected item and consequence before deciding.",
      "Choose Cancel, Escape, close, or the backdrop to dismiss without changing state.",
    ],
    style: [
      "DialogWindow is a focused native-decorated desktop window, sibling to ProjectHub, Workspace, Settings, and About.",
      "The confirmation dialog uses the shared raised graphite surface, strong border, and 14px radius.",
      "Cancel receives initial keyboard focus; the explicit action uses the danger treatment.",
    ],
    logic: [
      "The dialog is closed when the window opens and never appears automatically.",
      "Only an explicit click on the action button opens the secondary confirmation step.",
      "Confirming completes the illustrative action and closes the dialog.",
    ],
  },
};

export function DialogWindowConfirmation({ open, sessionTitle, onCancel, onConfirm }) {
  return (
    <ConfirmationDialog
      open={open}
      title="Archive this session?"
      description="It will leave the active session list, but can be reopened later."
      confirmLabel="Archive session"
      onCancel={onCancel}
      onConfirm={onConfirm}
      className="desktop-dialog-confirmation"
    >
      <div className="confirmation-dialog-target">
        <span>SESSION</span>
        <strong>{sessionTitle ?? "Provider migration review"}</strong>
      </div>
    </ConfirmationDialog>
  );
}

function DialogWindow() {
  const [dialogOpen, setDialogOpen] = useState(false);

  return (
    <NativeWindowFrame
      platform="macos"
      title="Confirm action"
      width="620px"
      height="420px"
      className="dialog-window-frame"
    >
      <div className="dialog-window-content">
        <span className="type-label">SECONDARY CONFIRMATION</span>
        <h2>Review before continuing</h2>
        <p>
          Consequential actions open in this focused window so the affected item and consequence
          stay clear before you commit.
        </p>
        <div className="dialog-window-target">
          <span>SESSION</span>
          <strong>Provider migration review</strong>
        </div>
        <Button variant="danger" onClick={() => setDialogOpen(true)}>
          Archive session…
        </Button>
      </div>
      <DialogWindowConfirmation
        open={dialogOpen}
        sessionTitle="Provider migration review"
        onCancel={() => setDialogOpen(false)}
        onConfirm={() => setDialogOpen(false)}
      />
    </NativeWindowFrame>
  );
}

export function DialogWindowPage() {
  const [guideOpen, setGuideOpen] = useState(false);

  return (
    <>
      <PageHeader
        title="DialogWindow"
        description="A focused desktop window for secondary confirmation before consequential actions."
        path="projects/desktop/dialog-window/"
      />
      <WindowSizeNote width="620" height="420" minimumWidth="480" minimumHeight="360" />
      <Section
        id="dialog-window"
        title="Secondary confirmation"
        description="The window opens without a dialog. Click the action to review and confirm it."
      >
        <WorkspaceGuideState
          className="dialog-window-guide-state"
          title="DialogWindow"
          description="A sibling desktop window that hosts confirmation dialogs on demand."
          guide={dialogGuide}
          side="right"
          open={guideOpen}
          onToggle={() => setGuideOpen((open) => !open)}
          onClose={() => setGuideOpen(false)}
        >
          <DialogWindow />
        </WorkspaceGuideState>
      </Section>
    </>
  );
}
