import { Icon } from "../../../../shared/Icon.jsx";
import { TrafficLights } from "../../../../shared/TrafficLights.jsx";

export function FocusedWorkspaceFrame({ title, children, className = "" }) {
  return (
    <div className={`workspace-focused-frame ${className}`}>
      <div className="workspace-focused-titlebar">
        <TrafficLights />
        <strong>suncode</strong>
        <span>{title}</span>
        <Icon name="settings" size={14} />
      </div>
      {children}
    </div>
  );
}
