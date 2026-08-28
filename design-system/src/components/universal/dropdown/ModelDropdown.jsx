import { useEffect, useId, useMemo, useRef, useState } from "react";
import { Icon } from "../../../shared/Icon.jsx";

export function ModelDropdown({ groups, initialValue, ariaLabel = "Model", className = "" }) {
  const initialGroup = useMemo(() => groups.find((group) => group.models.includes(initialValue)) ?? groups[0], [groups, initialValue]);
  const [open, setOpen] = useState(false);
  const [activeProvider, setActiveProvider] = useState(initialGroup.id);
  const [model, setModel] = useState(initialValue ?? initialGroup.models[0]);
  const rootRef = useRef(null);
  const triggerId = useId();
  const selectedGroup = groups.find((group) => group.id === activeProvider) ?? groups[0];

  useEffect(() => {
    const closeOnOutsideInteraction = (event) => {
      if (!rootRef.current?.contains(event.target)) setOpen(false);
    };
    const closeOnEscape = (event) => {
      if (event.key === "Escape") setOpen(false);
    };
    document.addEventListener("pointerdown", closeOnOutsideInteraction);
    document.addEventListener("keydown", closeOnEscape);
    return () => {
      document.removeEventListener("pointerdown", closeOnOutsideInteraction);
      document.removeEventListener("keydown", closeOnEscape);
    };
  }, []);

  return <div ref={rootRef} className={`dropdown-anchor model-dropdown ${className}`}>
    <button id={triggerId} type="button" className="dropdown-trigger dropdown-trigger-wide" aria-label={ariaLabel} aria-haspopup="menu" aria-expanded={open} aria-controls={`${triggerId}-menu`} onClick={() => setOpen(!open)}>
      <span>{model}</span><Icon name="chevron-right" size={12} className={open ? "is-open" : ""} />
    </button>
    {open && <div id={`${triggerId}-menu`} className="dropdown-menu dropdown-menu-model" role="menu" aria-label="Model provider">
      <div className="dropdown-menu-columns">
        <div className="dropdown-provider-list">
          {groups.map((group) => <button key={group.id} type="button" role="menuitem" aria-haspopup="menu" aria-expanded={activeProvider === group.id} className={`dropdown-provider ${activeProvider === group.id ? "is-active" : ""}`} onClick={() => setActiveProvider(group.id)}><span>{group.label}</span><Icon name="chevron-right" size={12} /></button>)}
        </div>
        <div className="dropdown-submenu" role="menu" aria-label={`${selectedGroup.label} models`}>
          {selectedGroup.models.map((option) => <button key={option} type="button" role="menuitemradio" aria-checked={model === option} className={`dropdown-option ${model === option ? "is-selected" : ""}`} onClick={() => { setModel(option); setOpen(false); }}><span>{option}</span>{model === option && <Icon name="check" size={13} />}</button>)}
        </div>
      </div>
    </div>}
  </div>;
}
