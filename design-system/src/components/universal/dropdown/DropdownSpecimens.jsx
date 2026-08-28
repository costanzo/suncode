import { useEffect, useId, useRef, useState } from "react";
import { Icon } from "../../../shared/Icon.jsx";
import { Specimen } from "../Specimen.jsx";

const effortOptions = ["Low", "Medium", "High"];
const modelGroups = [
  { id: "deepseek", label: "DeepSeek", models: ["DeepSeek V4 Flash", "DeepSeek V4 Pro"] },
  { id: "claude", label: "Claude", models: ["Claude Sonnet 5", "Claude Opus 5"] },
  { id: "openai", label: "OpenAI", models: ["GPT-5.5", "GPT-5.6 Sol"] },
];

function DropdownMenu({ label, children, className = "", id }) {
  return <div id={id} className={`dropdown-menu ${className}`} role="menu" aria-label={label}>{children}</div>;
}

function DropdownOption({ children, selected, onClick }) {
  return <button type="button" role="menuitemradio" aria-checked={selected} className={`dropdown-option ${selected ? "is-selected" : ""}`} onClick={onClick}>
    <span>{children}</span>
    {selected && <Icon name="check" size={13} />}
  </button>;
}

export function DropdownSpecimens() {
  const [effortOpen, setEffortOpen] = useState(false);
  const [effort, setEffort] = useState("Medium");
  const [modelOpen, setModelOpen] = useState(false);
  const [activeProvider, setActiveProvider] = useState("deepseek");
  const [model, setModel] = useState("DeepSeek V4 Pro");
  const rootRef = useRef(null);
  const effortId = useId();
  const modelId = useId();

  useEffect(() => {
    const handlePointerDown = (event) => {
      if (!rootRef.current?.contains(event.target)) {
        setEffortOpen(false);
        setModelOpen(false);
      }
    };
    const handleKeyDown = (event) => {
      if (event.key === "Escape") {
        setEffortOpen(false);
        setModelOpen(false);
      }
    };
    document.addEventListener("pointerdown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  const selectedGroup = modelGroups.find((group) => group.id === activeProvider);

  return <div ref={rootRef} className="specimen-grid specimen-grid-2 dropdown-specimens">
    <Specimen label="Single-level dropdown">
      <div className="dropdown-demo">
        <span className="field-label">Reasoning effort</span>
        <div className="dropdown-anchor">
          <button id={effortId} type="button" className="dropdown-trigger" aria-haspopup="menu" aria-expanded={effortOpen} aria-controls={`${effortId}-menu`} onClick={() => { setEffortOpen(!effortOpen); setModelOpen(false); }}>
            <span>{effort}</span><Icon name="chevron-right" size={12} className={effortOpen ? "is-open" : ""} />
          </button>
          {effortOpen && <DropdownMenu label="Reasoning effort" className="dropdown-menu-single" id={`${effortId}-menu`}>
            {effortOptions.map((option) => <DropdownOption key={option} selected={effort === option} onClick={() => { setEffort(option); setEffortOpen(false); }}>{option}</DropdownOption>)}
          </DropdownMenu>}
        </div>
        <p className="sample-note">One list of mutually exclusive options.</p>
      </div>
    </Specimen>

    <Specimen label="Two-level model dropdown">
      <div className="dropdown-demo">
        <span className="field-label">Model</span>
        <div className="dropdown-anchor">
          <button id={modelId} type="button" className="dropdown-trigger dropdown-trigger-wide" aria-haspopup="menu" aria-expanded={modelOpen} aria-controls={`${modelId}-menu`} onClick={() => { setModelOpen(!modelOpen); setEffortOpen(false); }}>
            <span>{model}</span><Icon name="chevron-right" size={12} className={modelOpen ? "is-open" : ""} />
          </button>
          {modelOpen && <DropdownMenu label="Model provider" className="dropdown-menu-model" id={`${modelId}-menu`}>
            <div className="dropdown-menu-columns">
              <div className="dropdown-provider-list">
                {modelGroups.map((group) => <button key={group.id} type="button" role="menuitem" aria-haspopup="menu" aria-expanded={activeProvider === group.id} className={`dropdown-provider ${activeProvider === group.id ? "is-active" : ""}`} onClick={() => setActiveProvider(group.id)}><span>{group.label}</span><Icon name="chevron-right" size={12} /></button>)}
              </div>
              <div className="dropdown-submenu" role="menu" aria-label={`${selectedGroup.label} models`}>
                {selectedGroup.models.map((option) => <DropdownOption key={option} selected={model === option} onClick={() => { setModel(option); setModelOpen(false); }}>{option}</DropdownOption>)}
              </div>
            </div>
          </DropdownMenu>}
        </div>
        <p className="sample-note">Provider groups reveal their model options.</p>
      </div>
    </Specimen>
  </div>;
}
