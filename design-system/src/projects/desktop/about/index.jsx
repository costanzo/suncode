import { useState } from "react";
import { PageHeader, Section } from "../../../shared/PagePrimitives.jsx";
import { NativeWindowFrame } from "../../../platforms/desktop/components/titlebar/index.js";
import logoUrl from "../../../assets/logos/suncode-logo-64.png";
import { WorkspaceGuideState } from "../workspace/WorkspaceGuide.jsx";
import { WindowSizeNote } from "../WindowSizeNote.jsx";

const aboutGuide = {
  tabs: {
    actions: [
      "Open About SunCode from the operating system application menu.",
      "Read the Desktop and Agent SDK versions beneath the SunCode identity.",
      "Close the native window to return control to the window that opened it.",
    ],
    style: [
      "The operating system owns the title bar, window controls, shadow, and outer resize behavior.",
      "The window is 420px wide by 320px high, with a 360px by 280px minimum size.",
      "The client content is centered with a 64px logo, a 22px product title, a compact two-row version list, and supporting text.",
    ],
    logic: [
      "About is a modal window; other SunCode windows remain unavailable until it closes.",
      "Only one About window exists at a time, and reopening the command activates the existing window.",
      "Desktop comes from the Avalonia application's installed version metadata; Agent SDK is queried asynchronously from the embedded Rust agent core through the SDK bindings.",
      "The Agent SDK row shows Loading while the query is in flight and Unavailable if the native version query fails.",
    ],
  },
};

export function AboutPage() {
  const [guideOpen, setGuideOpen] = useState(false);
  return (
    <>
      <PageHeader
        title="About"
        description="The native-decorated Avalonia window for product identity and component versions."
        path="projects/desktop/about/"
      />
      <WindowSizeNote width="420" height="320" minimumWidth="360" minimumHeight="280" />
      <Section
        id="about-window"
        title="About SunCode"
        description="The operating system supplies the title bar and window controls; the specimen represents the 420 × 320 client area with separate Desktop and Agent SDK version metadata."
      >
        <WorkspaceGuideState
          className="about-guide-state"
          title="About window"
          description="Review the product identity, component versions, native window treatment, and modal behavior."
          guide={aboutGuide}
          side="right"
          open={guideOpen}
          onToggle={() => setGuideOpen((open) => !open)}
          onClose={() => setGuideOpen(false)}
        >
          <NativeWindowFrame
            platform="macos"
            title="About SunCode"
            width="420px"
            height="348px"
            className="about-native-window"
          >
            <div className="about-window">
              <img src={logoUrl} alt="" />
              <strong>SunCode</strong>
              <div className="about-versions" aria-label="Component versions">
                <div className="about-version-row">
                  <span>Desktop</span>
                  <code>v0.0.1</code>
                </div>
                <div className="about-version-row">
                  <span>Agent SDK</span>
                  <code>v0.1.0</code>
                </div>
              </div>
              <span>General-purpose coding agent</span>
            </div>
          </NativeWindowFrame>
        </WorkspaceGuideState>
      </Section>
    </>
  );
}
