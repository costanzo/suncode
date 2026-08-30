import { useState } from "react";
import closeNormal from "../../../apps/desktop-avalonia/Assets/traffic-lights/1-close-1-normal.svg";
import closeHover from "../../../apps/desktop-avalonia/Assets/traffic-lights/2-close-2-hover.svg";
import closePress from "../../../apps/desktop-avalonia/Assets/traffic-lights/2-close-3-press.svg";
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

function TrafficLight({ kind, label, onClick }) {
  const [state, setState] = useState("normal");
  const sources = lightSources[kind];

  return (
    <button
      type="button"
      className={`traffic-light ${kind}`}
      aria-label={label}
      title={label}
      onClick={onClick}
      onPointerEnter={() => setState("hover")}
      onPointerLeave={() => setState("normal")}
      onPointerDown={() => setState("press")}
      onPointerUp={() => setState("hover")}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") setState("press");
      }}
      onKeyUp={() => setState("normal")}
      onBlur={() => setState("normal")}
    >
      <img src={sources[state]} alt="" draggable="false" />
    </button>
  );
}

export function TrafficLights({
  onClose,
  onMinimize,
  onMaximize,
  maximizeLabel = "Maximize window",
  className = "",
}) {
  return (
    <div className={`traffic-lights ${className}`} aria-label="Window controls">
      <TrafficLight kind="close" label="Close window" onClick={onClose} />
      <TrafficLight kind="minimize" label="Minimize window" onClick={onMinimize} />
      <TrafficLight kind="maximize" label={maximizeLabel} onClick={onMaximize} />
    </div>
  );
}
