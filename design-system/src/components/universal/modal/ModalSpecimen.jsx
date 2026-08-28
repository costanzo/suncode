import { useEffect, useRef, useState } from "react";
import { Icon } from "../../../shared/Icon.jsx";
import { Specimen } from "../Specimen.jsx";

export function ModalSpecimen() {
  const [dialogOpen, setDialogOpen] = useState(false);
  const dialogRef = useRef(null);

  useEffect(() => {
    if (!dialogOpen) return undefined;
    const priorFocus = document.activeElement;
    const dialog = dialogRef.current;
    const focusable = [...dialog.querySelectorAll('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])')];
    focusable[0]?.focus();
    const handleDialogKey = (event) => {
      if (event.key === "Escape") setDialogOpen(false);
      if (event.key !== "Tab" || focusable.length === 0) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
    };
    document.addEventListener("keydown", handleDialogKey);
    return () => { document.removeEventListener("keydown", handleDialogKey); priorFocus?.focus(); };
  }, [dialogOpen]);

  return <Specimen label="Focused dialog"><button className="btn btn-primary" onClick={() => setDialogOpen(true)}>Open undo dialog</button><p className="sample-note">Dialogs interrupt only protected, consequential decisions.</p>{dialogOpen && <div className="dialog-backdrop" role="presentation" onMouseDown={() => setDialogOpen(false)}><div ref={dialogRef} className="review-dialog" role="dialog" aria-modal="true" aria-labelledby="undo-dialog-title" aria-describedby="undo-dialog-description" onMouseDown={(event) => event.stopPropagation()}><div className="dialog-title"><div><h3 id="undo-dialog-title">Undo this turn?</h3><p id="undo-dialog-description">Four filesystem changes will be restored from the checkpoint.</p></div><button className="btn btn-icon btn-quiet" onClick={() => setDialogOpen(false)} aria-label="Close dialog"><Icon name="close" /></button></div><div className="dialog-list"><span><code>agent.rs</code><strong>Modified</strong></span><span><code>App.axaml</code><strong>Modified</strong></span></div><div className="dialog-actions"><button className="btn" onClick={() => setDialogOpen(false)}>Cancel</button><button className="btn btn-danger" onClick={() => setDialogOpen(false)}>Undo changes</button></div></div></div>}</Specimen>;
}
