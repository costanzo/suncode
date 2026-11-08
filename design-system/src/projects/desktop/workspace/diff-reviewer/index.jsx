import { useState } from "react";
import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { DiffReviewerWindow } from "./DiffReviewerWindow.jsx";
import "../styles/diff-reviewer.css";

export function DiffReviewerPage() {
  const [preview, setPreview] = useState("ready");
  const [platform, setPlatform] = useState("macos");
  const [constrained, setConstrained] = useState(false);
  const [open, setOpen] = useState(true);
  return (
    <>
      <PageHeader
        title="Diff Reviewer"
        description="A separate window for browsing changed files and reviewing each difference side by side."
      />
      <Section title="Change review window" id="diff-reviewer-window">
        <div className="diff-reviewer-preview-controls">
          <label>
            Preview state{" "}
            <select
              value={preview}
              onChange={(event) => {
                setPreview(event.target.value);
                setOpen(true);
              }}
            >
              {[
                ["ready", "Changes"],
                ["loading", "Loading"],
                ["error", "Read failure"],
                ["clean", "No changes"],
                ["unselected", "No selection"],
                ["binary", "Binary file"],
                ["limited", "Reading limit"],
              ].map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>
          <label>
            Window frame{" "}
            <select value={platform} onChange={(event) => setPlatform(event.target.value)}>
              <option value="macos">macOS</option>
              <option value="windows">Windows</option>
            </select>
          </label>
          <label className="diff-reviewer-width-toggle">
            <input
              type="checkbox"
              checked={constrained}
              onChange={(event) => setConstrained(event.target.checked)}
            />
            Constrained width
          </label>
        </div>
        <div className={`diff-reviewer-preview ${constrained ? "is-constrained" : ""}`}>
          {open ? (
            <DiffReviewerWindow
              key={preview}
              initialState={preview}
              platform={platform}
              onClose={() => setOpen(false)}
            />
          ) : (
            <div className="diff-reviewer-reopen">
              <button type="button" className="button button-neutral" onClick={() => setOpen(true)}>
                Open Diff reviewer
              </button>
            </div>
          )}
        </div>
        <p className="diff-reviewer-caption">
          Illustrative Git data. Select a file, expand unchanged lines, and use the arrows to move
          between difference blocks.
        </p>
      </Section>
      <Section title="Interaction contract" id="diff-reviewer-contract">
        <ul className="diff-reviewer-contract">
          <li>
            Source control opens one non-modal reviewer per project, carrying its Git range and
            selected file. Reopening activates that project’s reviewer.
          </li>
          <li>
            The changed-file tree contains changed paths and their parent folders. File selection
            opens the comparison at its first difference.
          </li>
          <li>
            Previous and next navigate contiguous difference blocks in the current file. Buttons
            disable at the first and last block.
          </li>
          <li>
            Both versions share row alignment and vertical scrolling. Unchanged ranges expand and
            collapse together. Long lines scroll horizontally.
          </li>
          <li>
            Refresh retains the selected file and expanded folders where possible. Closing the
            project ends the reviewer’s reads and window lifetime.
          </li>
        </ul>
      </Section>
    </>
  );
}
