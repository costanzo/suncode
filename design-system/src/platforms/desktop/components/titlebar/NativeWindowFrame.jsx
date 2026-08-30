import compactLogoUrl from "../../../../assets/logos/suncode-logo-small.svg";
import { Icon } from "../../../../shared/Icon.jsx";
import { TrafficLights } from "../../../../shared/TrafficLights.jsx";
import "./titlebar.css";

function WindowsWindowControls() {
  return (
    <div className="windows-window-controls" aria-label="Window controls">
      <button type="button" aria-label="Minimize window" title="Minimize">
        <Icon name="window-minimize" size={12} />
      </button>
      <button type="button" aria-label="Maximize window" title="Maximize">
        <Icon name="window-maximize" size={12} />
      </button>
      <button
        type="button"
        className="windows-window-close"
        aria-label="Close window"
        title="Close"
      >
        <Icon name="close" size={12} />
      </button>
    </div>
  );
}

export function NativeTitlebar({ platform, title, applicationName = "SunCode" }) {
  if (platform === "macos") {
    return (
      <div className="native-titlebar native-titlebar-macos">
        <TrafficLights />
        <strong>{title}</strong>
        <span className="native-titlebar-spacer" aria-hidden="true" />
      </div>
    );
  }

  return (
    <div className="native-titlebar native-titlebar-windows">
      <div className="windows-window-identity">
        <img src={compactLogoUrl} alt="" />
        <strong>{title}</strong>
        <span>{applicationName}</span>
      </div>
      <WindowsWindowControls />
    </div>
  );
}

export function NativeWindowFrame({
  platform = "macos",
  title,
  applicationName,
  width,
  height,
  children,
  className = "",
}) {
  const frameStyle = {
    ...(width ? { "--native-window-width": width } : {}),
    ...(height ? { "--native-window-height": height } : {}),
  };
  return (
    <div
      className={`native-window-frame is-${platform} ${height ? "has-fixed-height" : ""} ${className}`}
      style={frameStyle}
    >
      <NativeTitlebar platform={platform} title={title} applicationName={applicationName} />
      <div className="native-window-client">{children}</div>
    </div>
  );
}
