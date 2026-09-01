import { useState } from "react";
import { Specimen } from "../Specimen.jsx";
import { ConfirmationDialog } from "./ConfirmationDialog.jsx";

export function ModalSpecimen() {
  const [dialogOpen, setDialogOpen] = useState(false);

  return (
    <Specimen label="Confirmation dialog">
      <button className="btn btn-primary" onClick={() => setDialogOpen(true)}>
        Open archive dialog
      </button>
      <p className="sample-note">Use confirmation only for consequential actions.</p>
      <ConfirmationDialog
        open={dialogOpen}
        title="Archive this session?"
        description="It will leave the active session list, but can be reopened later."
        confirmLabel="Archive session"
        onCancel={() => setDialogOpen(false)}
        onConfirm={() => setDialogOpen(false)}
      >
        <div className="confirmation-dialog-target">
          <span>SESSION</span>
          <strong>Provider migration review</strong>
        </div>
      </ConfirmationDialog>
    </Specimen>
  );
}
