import { Specimen } from "../Specimen.jsx";

export function InputSpecimens() {
  return (
    <div className="specimen-grid specimen-grid-3">
      <Specimen label="Text input"><div className="stack"><label><span className="field-label">Session name</span><input className="field" defaultValue="Refactor provider cache" /></label><label><span className="field-label">Project path</span><input className="field" placeholder="/Users/name/project" /></label></div></Specimen>
      <Specimen label="Select and textarea"><div className="stack"><label><span className="field-label">Model</span><select className="field" defaultValue="deepseek"><option value="deepseek">DeepSeek V4 Pro</option><option value="claude">Claude Sonnet 5</option></select></label><label><span className="field-label">Instruction</span><textarea className="field" placeholder="Describe the change…" /></label></div></Specimen>
      <Specimen label="Validation"><label><span className="field-label">API endpoint</span><input className="field field-error" defaultValue="https://" aria-invalid="true" aria-describedby="endpoint-error" /><span className="field-help error" id="endpoint-error">Enter a complete HTTPS endpoint.</span></label></Specimen>
    </div>
  );
}
