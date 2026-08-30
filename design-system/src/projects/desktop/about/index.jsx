import { useState } from "react";
import { PageHeader, Section } from "../../../shared/PagePrimitives.jsx";
import logoUrl from "../../../assets/logos/suncode-logo-64.png";
import { WorkspaceGuideState } from "../workspace/WorkspaceGuide.jsx";

const aboutGuide = { tabs: {
  actions: ["Open About SunCode from the operating system application menu.", "Read the installed product version beneath the SunCode identity.", "Close the native window to return control to the window that opened it."],
  style: ["The operating system owns the title bar, window controls, shadow, and outer resize behavior.", "The window is 420px wide by 320px high, with a 360px by 280px minimum size.", "The client content is centered with a 64px logo, 12px vertical gaps, a 22px product title, and compact supporting text."],
  logic: ["About is a modal window; other SunCode windows remain unavailable until it closes.", "Only one About window exists at a time, and reopening the command activates the existing window.", "The displayed version comes from the desktop application's installed version metadata."],
} };

export function AboutPage() {
  const [guideOpen, setGuideOpen] = useState(false);
  return <><PageHeader title="About" description="The native-decorated Avalonia window for product identity and installed version." path="projects/desktop/about/" /><Section id="about-window" title="About SunCode" description="The operating system supplies the title bar and window controls; the specimen begins at the client area."><WorkspaceGuideState className="about-guide-state" title="About window" description="Review the product identity, installed version, native window treatment, and modal behavior." guide={aboutGuide} side="right" open={guideOpen} onToggle={() => setGuideOpen((open) => !open)} onClose={() => setGuideOpen(false)}><div className="about-window"><img src={logoUrl} alt="" /><strong>SunCode</strong><code>Version 0.0.1</code><span>General-purpose coding agent</span></div></WorkspaceGuideState></Section></>;
}
