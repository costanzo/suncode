"use client";

import { useId, useState } from "react";
import { Icon } from "../../shared/Icon.jsx";
import { WorkspaceWindow } from "../desktop/workspace/shell/WorkspaceShell.jsx";
import "./workbench.css";

const steps = [
  {
    label: "Inspect the project",
    heading: "Understand before changing",
    text: "Explore project files, follow tool activity, and keep the conversation in view.",
    conversation: "content-updating",
    progress:
      "I’m reading the desktop window components and their shared styles before changing the conversation layout.",
    review: "running-no-changes",
    drawer: null,
  },
  {
    label: "Review the scope",
    heading: "Decide with the details in view",
    text: "Review the build command in the opened project before allowing it to run.",
    conversation: "content-thinking",
    progress:
      "The layout changes are ready. The design build needs your approval before it can run in this project.",
    review: "approval",
    drawer: null,
  },
  {
    label: "Inspect the changes",
    heading: "See what changed",
    text: "Browse the example diff. Checkpoint undo covers file edits, with conflict checks. Commands and external effects have separate boundaries.",
    conversation: "content-waiting",
    progress: "The layout changes are complete.",
    review: "completed",
    drawer: "git",
  },
];

export function WorkbenchDemo() {
  const previewId = useId();
  const [step, setStep] = useState(0);
  const [notice, setNotice] = useState("");
  const current = steps[step];
  const showWindowAction = (action) => setNotice(`${action} selected in the illustrative window.`);
  const showDecision = (decision) => {
    const messages = {
      "allow-once": "Example decision recorded. This build would be allowed once.",
      "allow-session": "Example decision recorded. This build would be allowed for this session.",
      deny: "Example decision recorded. This build would be denied.",
    };
    setNotice(messages[decision]);
  };

  return (
    <div className="website-workbench" data-step={step}>
      <div className="website-workbench-steps" aria-label="Explore the illustrative workflow">
        {steps.map((item, index) => (
          <button
            type="button"
            key={item.label}
            aria-pressed={step === index}
            aria-controls={previewId}
            onClick={() => {
              setStep(index);
              setNotice("");
            }}
          >
            <Icon name={index === step ? "check" : "chevron-right"} size={14} />
            {item.label}
          </button>
        ))}
      </div>
      <div className="website-workbench-label">
        <span>Illustrative SunCode desktop session</span>
        <span>Explore the panels</span>
      </div>
      <div
        className="website-workbench-window"
        id={previewId}
        role="region"
        aria-label={`${current.label} · illustrative desktop window`}
      >
        <WorkspaceWindow
          key={step}
          titlebarPlatform="ubuntu"
          conversationState={current.conversation}
          reviewState={current.review}
          conversationExample={{
            prompt: "Improve the desktop conversation layout and verify the design build.",
            progress: current.progress,
            response:
              "Updated the conversation layout, preserved attachment behavior, and checked the design build. You can inspect the changed files and diff below.",
            tools: [{ icon: "files", title: "Read desktop window components", tone: "running" }],
          }}
          statusbarContent={
            <>
              <div>
                <code>main</code>
                <span>Local project</span>
              </div>
              <div>
                <code>Illustrative data</code>
                <code>Rust agent · native desktop</code>
              </div>
            </>
          }
          initialDrawer={current.drawer}
          initialChangesOpen={step === 2}
          onApprovalDecision={showDecision}
          onWindowMinimize={() => showWindowAction("Minimize")}
          onWindowMaximize={() => showWindowAction("Maximize")}
          onWindowClose={() => showWindowAction("Close")}
          onTitlebarCommand={showWindowAction}
        />
      </div>
      <div className="website-workbench-caption" aria-live="polite">
        <h3>{current.heading}</h3>
        <p>{current.text}</p>
      </div>
      <p className="website-workbench-notice" role="status">
        {notice || "Sample data. All interactions stay within this preview."}
      </p>
    </div>
  );
}
