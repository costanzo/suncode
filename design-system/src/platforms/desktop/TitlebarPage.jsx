import { Icon } from "../../shared/Icon.jsx";
import { PageHeader, Section } from "../../shared/PagePrimitives.jsx";
import { NativeWindowFrame } from "./components/titlebar/index.js";

function WindowTemplateContent() {
  return <div className="native-window-template-content">
    <aside className="native-window-template-sidebar"><strong>SunCode</strong><span>Overview</span><span>Sessions</span><span>Settings</span></aside>
    <main className="native-window-template-main"><header><strong>Window content</strong><span>Native desktop frame</span></header><div className="native-window-template-canvas"><Icon name="workspace" size={26} /><strong>Application client area</strong><span>Product navigation, toolbars, and task content begin below the platform-owned title bar.</span></div></main>
  </div>;
}

const specimens = [
  { platform: "macos", name: "macOS", description: "Traffic lights lead the title bar; the window title remains optically centered." },
  { platform: "windows", name: "Windows", description: "Application identity leads; minimize, maximize, and close actions occupy the trailing edge." },
];

export function DesktopTitlebarPage() {
  return <><PageHeader title="Titlebar" description="Reusable native window frames for macOS and Windows desktop surfaces." /><Section id="native-window-frames" title="Platform window templates" description="Use the operating-system variant that matches the host; product content starts inside the shared client-area slot."><div className="native-window-specimens">{specimens.map((specimen) => <article className="native-window-specimen" key={specimen.platform}><div className="native-window-specimen-heading"><div><strong>{specimen.name}</strong><span>{specimen.description}</span></div><code>760 × 440</code></div><NativeWindowFrame platform={specimen.platform} title={specimen.platform === "macos" ? "Welcome to SunCode" : "SunCode — suncode"} width="760px" height="440px"><WindowTemplateContent /></NativeWindowFrame></article>)}</div></Section></>;
}
