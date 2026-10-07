import { useState } from "react";
import closeNormal from "../../../apps/desktop-avalonia/Assets/traffic-lights/1-close-1-normal.svg";
import closeHover from "../../../apps/desktop-avalonia/Assets/traffic-lights/2-close-2-hover.svg";
import closePress from "../../../apps/desktop-avalonia/Assets/traffic-lights/2-close-3-press.svg";
import noFocus from "../../../apps/desktop-avalonia/Assets/traffic-lights/0-all-three-nofocus.svg";
import minimizeNormal from "../../../apps/desktop-avalonia/Assets/traffic-lights/2-minimize-1-normal.svg";
import minimizeHover from "../../../apps/desktop-avalonia/Assets/traffic-lights/2-minimize-2-hover.svg";
import minimizePress from "../../../apps/desktop-avalonia/Assets/traffic-lights/2-minimize-3-press.svg";
import maximizeNormal from "../../../apps/desktop-avalonia/Assets/traffic-lights/3-maximize-1-normal.svg";
import maximizeHover from "../../../apps/desktop-avalonia/Assets/traffic-lights/3-maximize-2-hover.svg";
import maximizePress from "../../../apps/desktop-avalonia/Assets/traffic-lights/3-maximize-3-press.svg";

const lightSources = {
  close: { normal: closeNormal, hover: closeHover, press: closePress },
  minimize: { normal: minimizeNormal, hover: minimizeHover, press: minimizePress },
  maximize: { normal: maximizeNormal, hover: maximizeHover, press: maximizePress },
};

function TrafficLight({ kind, label, onClick, isActive }) {
  const [state, setState] = useState("normal");
  const sources = lightSources[kind];
  const source = isActive ? sources[state] : noFocus;

  return (
    <button
      type="button"
      className={`traffic-light ${kind}`}
      aria-label={label}
      title={label}
      onClick={onClick}
      onPointerEnter={() => isActive && setState("hover")}
      onPointerLeave={() => isActive && setState("normal")}
      onPointerDown={() => isActive && setState("press")}
      onPointerUp={() => isActive && setState("hover")}
      onKeyDown={(event) => {
        if (isActive && (event.key === "Enter" || event.key === " ")) setState("press");
      }}
      onKeyUp={() => isActive && setState("normal")}
      onBlur={() => setState("normal")}
    >
      <img src={source} alt="" draggable="false" />
    </button>
  );
}

export function TrafficLights({
  onClose,
  onMinimize,
  onMaximize,
  maximizeLabel = "Maximize window",
  isActive = true,
  className = "",
}) {
  return (
    <div className={`traffic-lights ${className}`} aria-label="Window controls">
      <TrafficLight kind="close" label="Close window" onClick={onClose} isActive={isActive} />
      <TrafficLight kind="minimize" label="Minimize window" onClick={onMinimize} isActive={isActive} />
      <TrafficLight kind="maximize" label={maximizeLabel} onClick={onMaximize} isActive={isActive} />
    </div>
  );
}
