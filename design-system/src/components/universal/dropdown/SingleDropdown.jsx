import { useEffect, useId, useRef, useState } from "react";
import { Icon } from "../../../shared/Icon.jsx";

function normalizeOption(option) {
  if (typeof option === "string") return { value: option, label: option };
  return { value: option.value, label: option.label ?? option.value };
}

export function SingleDropdown({ options, initialValue, value, onChange, ariaLabel = "Select an option", className = "", menuClassName = "", wide = false }) {
  const normalizedOptions = options.map(normalizeOption);
  const fallbackValue = normalizedOptions[0]?.value ?? "";
  const isControlled = value !== undefined;
  const [internalValue, setInternalValue] = useState(initialValue ?? fallbackValue);
  const [open, setOpen] = useState(false);
  const rootRef = useRef(null);
  const triggerId = useId();
  const selectedValue = isControlled ? value : internalValue;
  const selectedOption = normalizedOptions.find((option) => option.value === selectedValue) ?? normalizedOptions[0];

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

  const selectOption = (nextValue) => {
    if (!isControlled) setInternalValue(nextValue);
    onChange?.(nextValue);
    setOpen(false);
  };

  return <div ref={rootRef} className={`dropdown-anchor single-dropdown ${className}`}>
    <button id={triggerId} type="button" className={`dropdown-trigger ${wide ? "dropdown-trigger-wide" : ""}`} aria-label={ariaLabel} aria-haspopup="menu" aria-expanded={open} aria-controls={`${triggerId}-menu`} onClick={() => setOpen((current) => !current)}>
      <span>{selectedOption?.label ?? "Select"}</span><Icon name="chevron-right" size={12} className={open ? "is-open" : ""} />
    </button>
    {open && <div id={`${triggerId}-menu`} className={`dropdown-menu dropdown-menu-single ${menuClassName}`} role="menu" aria-label={ariaLabel}>
      {normalizedOptions.map((option) => <button key={option.value} type="button" role="menuitemradio" aria-checked={selectedValue === option.value} className={`dropdown-option ${selectedValue === option.value ? "is-selected" : ""}`} onClick={() => selectOption(option.value)}><span>{option.label}</span>{selectedValue === option.value && <Icon name="check" size={13} />}</button>)}
    </div>}
  </div>;
}
