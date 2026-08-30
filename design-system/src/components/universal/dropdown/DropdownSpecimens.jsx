import { Specimen } from "../Specimen.jsx";
import { ModelDropdown } from "./ModelDropdown.jsx";
import { SingleDropdown } from "./SingleDropdown.jsx";

const effortOptions = ["Low", "Medium", "High"];
const modelGroups = [
  { id: "deepseek", label: "DeepSeek", models: ["DeepSeek V4 Flash", "DeepSeek V4 Pro"] },
  { id: "claude", label: "Claude", models: ["Claude Sonnet 5", "Claude Opus 5"] },
  { id: "openai", label: "OpenAI", models: ["GPT-5.5", "GPT-5.6 Sol"] },
];

export function DropdownSpecimens() {
  return (
    <div className="specimen-grid specimen-grid-2 dropdown-specimens">
      <Specimen label="Single-level dropdown">
        <div className="dropdown-demo">
          <span className="field-label">Reasoning effort</span>
          <SingleDropdown
            options={effortOptions}
            initialValue="Medium"
            ariaLabel="Reasoning effort"
          />
          <p className="sample-note">One list of mutually exclusive options.</p>
        </div>
      </Specimen>

      <Specimen label="Two-level model dropdown">
        <div className="dropdown-demo">
          <span className="field-label">Model</span>
          <ModelDropdown groups={modelGroups} initialValue="DeepSeek V4 Pro" ariaLabel="Model" />
          <p className="sample-note">Provider groups reveal their model options.</p>
        </div>
      </Specimen>
    </div>
  );
}
