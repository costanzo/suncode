import { useState } from "react";
import { Specimen } from "../Specimen.jsx";

export function ToggleSpecimen() {
  const [enabled, setEnabled] = useState(true);
  return <Specimen label="Toggle"><label className="toggle"><input type="checkbox" checked={enabled} onChange={(event) => setEnabled(event.target.checked)} /><span className="toggle-track" /> Provider {enabled ? "enabled" : "disabled"}</label></Specimen>;
}
