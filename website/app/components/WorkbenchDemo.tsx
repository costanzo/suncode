"use client";

import { useState } from "react";
import { BrandMark } from "./BrandMark";
import {
  ArrowDown,
  ArrowUp,
  Check,
  ChevronRight,
  FileCode2,
  Folder,
  GitBranch,
  LockKeyhole,
  MessageSquare,
  PanelLeft,
  Terminal,
  Undo2,
} from "lucide-react";

const steps = [
  {
    label: "Inspect the project",
    heading: "Understand before changing",
    text: "Project context, visible tools, and a conversation that stays with the work.",
  },
  {
    label: "Review the scope",
    heading: "Decide with the details in view",
    text: "Sensitive operations pause for your decision. Inspect the action and its scope.",
  },
  {
    label: "Inspect the changes",
    heading: "See what changed",
    text: "Review checkpointed file mutations and restore them with conflict checks.",
  },
];

export function WorkbenchDemo() {
  const [step, setStep] = useState(0);
  return (
    <div className="demo-section">
      <div className="demo-tabs" aria-label="Explore the illustrative workflow">
        {steps.map((item, index) => (
          <button
            type="button"
            key={item.label}
            aria-pressed={step === index}
            aria-controls="workbench-preview"
            onClick={() => setStep(index)}
          >
            <span className="step-mark">
              {index < step ? <Check size={13} /> : <span />}
            </span>
            {item.label}
          </button>
        ))}
      </div>
      <div
        className="workbench"
        id="workbench-preview"
        role="region"
        aria-label="Illustrative SunCode desktop session"
      >
        <div className="workbench-titlebar">
          <div className="window-dots" aria-hidden="true">
            <i />
            <i />
            <i />
          </div>
          <span className="demo-project">
            <Folder size={13} /> suncode <ChevronRight size={12} />
            <span>Improve request recovery</span>
          </span>
          <span className="illustration-label">Illustrative session</span>
        </div>
        <div className="workbench-body">
          <aside className="demo-sidebar">
            <div className="demo-sidebar-heading">
              <PanelLeft size={15} />
              <span>Sessions</span>
            </div>
            <div className="demo-session selected">
              <MessageSquare size={14} />
              <span>
                Improve request recovery<small>Current session</small>
              </span>
            </div>
            <div className="demo-session">
              <MessageSquare size={14} />
              <span>Review API boundaries</span>
            </div>
            <div className="demo-session">
              <MessageSquare size={14} />
              <span>Explain the data layer</span>
            </div>
            <div className="demo-files">
              <div className="demo-sidebar-heading">Project</div>
              <div>
                <Folder size={13} /> agent <ChevronRight size={11} />
              </div>
              <div className="indent">
                <Folder size={13} /> crates
              </div>
              <div className="indent file">
                <FileCode2 size={13} /> retry.rs
              </div>
              <div className="indent file">
                <FileCode2 size={13} /> retry_tests.rs
              </div>
            </div>
            <div className="demo-branch">
              <GitBranch size={13} /> feature/request-recovery
            </div>
          </aside>
          <div className="demo-conversation">
            <div className="demo-conversation-heading">
              <h3>Improve request recovery</h3>
              <span>Claude</span>
            </div>
            <div className="demo-user">
              Handle temporary provider failures and add focused tests.
            </div>
            <div className="demo-assistant">
              <BrandMark size={22} label="SunCode" />
              <div>
                <p>
                  I found the request boundary. I’ll add a bounded retry before
                  any response content is received, keeping cancellation active.
                </p>
                <div className="demo-tool">
                  <Check size={13} />
                  <span>Read request and cancellation paths</span>
                </div>
              </div>
            </div>
            <div className="demo-stage" aria-live="polite">
              {step === 0 && (
                <div className="demo-inspection">
                  <div>
                    <FileCode2 size={15} />
                    <strong>retry.rs</strong>
                    <span>Project context</span>
                  </div>
                  <pre>
                    <code>
                      <span className="code-muted">
                        // Keep retries behind the request boundary.
                      </span>
                      {"\n"}
                      <span className="code-keyword">
                        if
                      </span> response_started {"{"}
                      {"\n"} <span className="code-keyword">return</span>{" "}
                      Err(error);{"\n"}
                      {"}"}
                    </code>
                  </pre>
                  <p>Reading the existing behavior before making a change.</p>
                </div>
              )}
              {step === 1 && (
                <div className="demo-approval">
                  <div className="demo-approval-title">
                    <LockKeyhole size={17} />
                    <strong>Approval needed</strong>
                    <span>Process execution</span>
                  </div>
                  <p>Run focused tests in the opened project.</p>
                  <code>cargo test -p suncode-core retry</code>
                  <div className="demo-decision">
                    <span>Scope: current project</span>
                    <span>Awaiting your decision</span>
                  </div>
                </div>
              )}
              {step === 2 && (
                <div className="demo-diff">
                  <div>
                    <FileCode2 size={15} />
                    <strong>retry.rs</strong>
                    <span>Checkpointed edit</span>
                  </div>
                  <pre>
                    <code>
                      <span className="diff-context">
                        {" "}
                        match request.send().await {"{"}
                      </span>
                      {"\n"}
                      <span className="diff-added">
                        + Err(error) if can_retry(&error) =&gt; {"{"}
                      </span>
                      {"\n"}
                      <span className="diff-added">
                        + wait_with_cancellation(delay).await?;
                      </span>
                      {"\n"}
                      <span className="diff-added">+ {"}"}</span>
                      {"\n"}
                      <span className="diff-context">
                        {" "}
                        result =&gt; return result,
                      </span>
                    </code>
                  </pre>
                  <div className="demo-checkpoint">
                    <Undo2 size={14} />
                    File checkpoint available<span>Conflict-aware restore</span>
                  </div>
                </div>
              )}
            </div>
            <div className="demo-composer">
              <span>Give the next instruction…</span>
              <span className="composer-send" aria-hidden="true">
                <ArrowUp size={15} />
              </span>
            </div>
          </div>
          <aside className="demo-review">
            <h4>Review</h4>
            <div className="demo-review-label">Current turn</div>
            <div className="review-summary">
              <span className="state-dot" />
              {step === 0
                ? "Inspecting project"
                : step === 1
                  ? "Waiting for approval"
                  : "Changes ready"}
            </div>
            <div className="review-rule" />
            <div className="demo-review-label">Files</div>
            <div className="review-file">
              <FileCode2 size={13} /> retry.rs
              <span>{step === 2 ? "+3" : "—"}</span>
            </div>
            <div className="review-file">
              <FileCode2 size={13} /> retry_tests.rs
              <span>{step === 2 ? "+12" : "—"}</span>
            </div>
            <div className="review-rule" />
            <div className="demo-review-label">Tool activity</div>
            <div className="review-activity">
              <Check size={13} /> Read files
            </div>
            <div className="review-activity">
              <Terminal size={13} />{" "}
              {step < 2 ? "Tests pending" : "Focused tests passed"}
            </div>
            <p className="demo-review-note">
              Checkpoint undo covers file edits. Commands and external effects
              have separate boundaries.
            </p>
          </aside>
        </div>
        <div className="workbench-status">
          <span>
            <span className="state-dot" /> Local project
          </span>
          <span>Rust agent · native desktop</span>
          <span>
            <ArrowUp size={11} /> 0 B/s <ArrowDown size={11} /> 0 B/s
          </span>
        </div>
      </div>
      <div className="demo-caption" aria-live="polite">
        <h3>{steps[step].heading}</h3>
        <p>{steps[step].text}</p>
      </div>
    </div>
  );
}
