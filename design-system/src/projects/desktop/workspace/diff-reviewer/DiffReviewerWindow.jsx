import { useEffect, useRef, useState } from "react";
import { NativeWindowFrame } from "../../../../platforms/desktop/components/titlebar/index.js";
import { Icon } from "../../../../shared/Icon.jsx";
import { ChangedFileTree } from "./ChangedFileTree.jsx";
import { DiffDocument } from "./DiffDocument.jsx";
import { comparisons, comparisonsByRange, comparisonSides } from "./data.js";

function Action({ icon, label, disabled, onClick }) {
  return (
    <button
      type="button"
      className="workspace-icon-button"
      title={label}
      aria-label={label}
      disabled={disabled}
      onClick={onClick}
    >
      <Icon name={icon} size={15} />
    </button>
  );
}

export function DiffReviewerWindow({ initialState = "ready", platform = "macos", onClose }) {
  const [scope, setScope] = useState("all");
  const [state, setState] = useState(initialState);
  const [selectedPath, setSelectedPath] = useState(
    initialState === "binary" ? comparisons.at(-1).path : comparisons[0].path,
  );
  const [activeChange, setActiveChange] = useState(0);
  const [expanded, setExpanded] = useState(new Set());
  const [treeWidth, setTreeWidth] = useState(null);
  const beforePane = useRef(null);
  const afterPane = useRef(null);
  const sidebar = useRef(null);
  const body = useRef(null);
  const drag = useRef(null);
  const refreshTimer = useRef(null);
  const navigationFrame = useRef(null);
  const navigating = useRef(false);
  const initialChange = useRef(0);
  useEffect(
    () => () => {
      clearTimeout(refreshTimer.current);
      cancelAnimationFrame(navigationFrame.current);
    },
    [],
  );

  const files = state === "clean" ? [] : comparisonsByRange[scope];
  const file = state === "unselected" ? null : files.find((item) => item.path === selectedPath);
  const displayed =
    state === "limited" && file ? { ...file, blocks: file.blocks.slice(0, 3), changes: 1 } : file;
  const readable =
    displayed &&
    !displayed.binary &&
    (state === "ready" || state === "binary" || state === "limited");
  const [leftTitle, rightTitle] = comparisonSides[scope];

  function selectFile(path) {
    clearTimeout(refreshTimer.current);
    setSelectedPath(path);
    setActiveChange(0);
    initialChange.current = 0;
    setExpanded(new Set());
    setState("ready");
  }

  function changeScope(value) {
    setScope(value);
    const nextFiles = comparisonsByRange[value];
    selectFile(nextFiles.find((item) => item.path === selectedPath)?.path ?? nextFiles[0]?.path);
    if (state === "clean") setState("clean");
  }

  function navigate(index) {
    navigating.current = true;
    cancelAnimationFrame(navigationFrame.current);
    for (const pane of [beforePane.current, afterPane.current]) {
      const target = pane?.querySelector(`[data-change="${index}"]`);
      if (target)
        pane.scrollTop += target.getBoundingClientRect().top - pane.getBoundingClientRect().top;
    }
    setActiveChange(index);
    // Scroll tracking resumes after both panes have dispatched their programmatic scrolls.
    navigationFrame.current = requestAnimationFrame(() => {
      navigationFrame.current = requestAnimationFrame(() => {
        navigating.current = false;
      });
    });
  }

  function syncScroll(event, other) {
    const pane = event.currentTarget;
    if (other.current && Math.abs(other.current.scrollTop - pane.scrollTop) > 1)
      other.current.scrollTop = pane.scrollTop;
    if (navigating.current) return;
    const changes = [...pane.querySelectorAll("[data-change]")];
    const marker = pane.getBoundingClientRect().top + pane.clientHeight / 3;
    const current =
      changes.filter((item) => item.getBoundingClientRect().top <= marker).at(-1) ?? changes[0];
    if (current) setActiveChange(Number(current.dataset.change));
  }

  function toggleContext(id) {
    setExpanded((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  function resizeTree(width) {
    const styles = getComputedStyle(body.current);
    const minimum = parseFloat(styles.getPropertyValue("--diff-reviewer-tree-min-width"));
    const maximum = Math.min(
      parseFloat(styles.getPropertyValue("--diff-reviewer-tree-max-width")),
      body.current.clientWidth -
        parseFloat(styles.getPropertyValue("--diff-reviewer-divider-width")) -
        2 * parseFloat(styles.getPropertyValue("--diff-reviewer-pane-min-width")),
    );
    setTreeWidth(Math.max(minimum, Math.min(maximum, width)));
  }

  function refresh() {
    const settled = ["error", "loading"].includes(state) ? "ready" : state;
    initialChange.current = activeChange;
    setState("loading");
    clearTimeout(refreshTimer.current);
    refreshTimer.current = setTimeout(() => {
      setState(settled);
    }, 450);
  }

  useEffect(() => {
    if (readable) navigate(Math.min(initialChange.current, displayed.changes - 1));
    // A new document starts at its first difference. Folding preserves the current document.
  }, [selectedPath, scope, readable]);

  let message;
  if (state === "loading")
    message = ["activity", "Reading changes…", "Loading the selected Git comparison."];
  else if (state === "error")
    message = [
      "warning",
      "Could not read this comparison",
      "Refresh to retry the repository read.",
    ];
  else if (state === "clean")
    message = ["check", "No changes", "There are no changed files in this Git range."];
  else if (!file)
    message = [
      "files",
      "Select a changed file",
      "Choose a file in the tree to compare its versions.",
    ];
  else if (file.binary)
    message = ["file", "Binary file", "This file has changes that cannot be displayed as text."];

  return (
    <NativeWindowFrame
      platform={platform}
      title="suncode — Diff reviewer"
      className="diff-reviewer-window"
      height="var(--diff-reviewer-height)"
      onClose={onClose}
    >
      <section className="diff-reviewer" aria-label="Diff reviewer">
        <header className="diff-reviewer-toolbar">
          <div className="workspace-scope" role="group" aria-label="Git range">
            {[
              ["all", "All"],
              ["staged", "Staged"],
              ["unstaged", "Unstaged"],
            ].map(([value, label]) => (
              <button
                key={value}
                type="button"
                aria-pressed={scope === value}
                className={scope === value ? "is-selected" : ""}
                onClick={() => changeScope(value)}
              >
                {label}
              </button>
            ))}
          </div>
          <span className="diff-reviewer-branch">
            <Icon name="git" size={14} /> main
          </span>
          <Action
            icon="refresh"
            label="Refresh changes"
            onClick={refresh}
            disabled={state === "loading"}
          />
        </header>
        <div
          className="diff-reviewer-body"
          ref={body}
          style={treeWidth ? { "--diff-reviewer-tree-width": `${treeWidth}px` } : undefined}
        >
          <aside className="diff-reviewer-sidebar" ref={sidebar}>
            <header>
              <strong>Changes</strong>
              <span>{files.length} files</span>
            </header>
            <ChangedFileTree files={files} selectedPath={file?.path} onSelect={selectFile} />
          </aside>
          <div
            className="diff-reviewer-divider"
            role="separator"
            aria-label="Resize changed files"
            aria-orientation="vertical"
            aria-valuemin="180"
            aria-valuemax="360"
            aria-valuenow={treeWidth ?? 240}
            tabIndex={0}
            onPointerDown={(event) => {
              drag.current = { x: event.clientX, width: sidebar.current.clientWidth };
              event.currentTarget.setPointerCapture(event.pointerId);
            }}
            onPointerMove={(event) => {
              if (drag.current) resizeTree(drag.current.width + event.clientX - drag.current.x);
            }}
            onPointerUp={() => {
              drag.current = null;
            }}
            onPointerCancel={() => {
              drag.current = null;
            }}
            onKeyDown={(event) => {
              if (["ArrowLeft", "ArrowRight"].includes(event.key)) {
                event.preventDefault();
                resizeTree(sidebar.current.clientWidth + (event.key === "ArrowRight" ? 12 : -12));
              }
            }}
          />
          <section className="diff-reviewer-detail" aria-label="Selected file comparison">
            <header className="diff-reviewer-file-toolbar">
              <span className="diff-reviewer-path" title={file?.path}>
                {file?.path ?? "Diff reviewer"}
              </span>
              {file && !file.binary && (
                <span className="diff-reviewer-stats">
                  <b>+{file.additions}</b>
                  <i>−{file.deletions}</i>
                </span>
              )}
              <nav aria-label="Difference navigation">
                <Action
                  icon="arrow-up"
                  label="Previous difference"
                  disabled={!readable || activeChange === 0}
                  onClick={() => navigate(activeChange - 1)}
                />
                <output aria-live="polite" aria-label="Current difference">
                  {readable ? `${activeChange + 1} / ${displayed.changes}` : "0 / 0"}
                </output>
                <Action
                  icon="arrow-down"
                  label="Next difference"
                  disabled={!readable || activeChange >= displayed.changes - 1}
                  onClick={() => navigate(activeChange + 1)}
                />
              </nav>
            </header>
            {file?.oldPath && (
              <div className="diff-reviewer-note">
                Renamed from <code>{file.oldPath}</code>
              </div>
            )}
            {state === "limited" && (
              <div className="diff-reviewer-note is-warning" role="status">
                Preview truncated at the reading limit. Navigation covers the loaded differences.
              </div>
            )}
            {message ? (
              <div
                className={`diff-reviewer-message ${state === "error" ? "is-error" : ""}`}
                role="status"
              >
                <Icon name={message[0]} size={24} />
                <strong>{message[1]}</strong>
                <span>{message[2]}</span>
                {state === "error" && (
                  <button type="button" className="button button-neutral compact" onClick={refresh}>
                    Retry
                  </button>
                )}
              </div>
            ) : (
              <>
                <div className="diff-reviewer-pane-headings">
                  <span>{leftTitle}</span>
                  <span>{rightTitle}</span>
                </div>
                <div className="diff-reviewer-documents" key={`${scope}-${selectedPath}`}>
                  <div
                    ref={beforePane}
                    className="diff-reviewer-document"
                    role="region"
                    aria-label="Old version"
                    tabIndex={0}
                    onScroll={(event) => syncScroll(event, afterPane)}
                  >
                    <DiffDocument
                      file={displayed}
                      side="before"
                      expanded={expanded}
                      activeChange={activeChange}
                      onToggleContext={toggleContext}
                    />
                  </div>
                  <div
                    ref={afterPane}
                    className="diff-reviewer-document"
                    role="region"
                    aria-label="New version"
                    tabIndex={0}
                    onScroll={(event) => syncScroll(event, beforePane)}
                  >
                    <DiffDocument
                      file={displayed}
                      side="after"
                      expanded={expanded}
                      activeChange={activeChange}
                      onToggleContext={toggleContext}
                    />
                  </div>
                </div>
              </>
            )}
          </section>
        </div>
      </section>
    </NativeWindowFrame>
  );
}
