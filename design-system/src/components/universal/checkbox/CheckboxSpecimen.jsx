import { Specimen } from "../Specimen.jsx";

export function CheckboxSpecimen() {
  return (
    <Specimen label="Checkbox">
      <div className="stack">
        <label className="checkbox">
          <input type="checkbox" defaultChecked /> Save as default
        </label>
        <label className="checkbox">
          <input type="checkbox" /> Include diagnostics
        </label>
        <label className="checkbox">
          <input type="checkbox" disabled /> Managed by policy
        </label>
      </div>
    </Specimen>
  );
}
