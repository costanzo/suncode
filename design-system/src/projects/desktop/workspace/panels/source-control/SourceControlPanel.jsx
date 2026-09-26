import { useState } from "react";

import { Icon } from "../../../../../shared/Icon.jsx";

import { ContextUsage } from "../../shared/ContextUsage.jsx";
import { IconButton } from "../../shared/IconButton.jsx";
import { TurnChangeSummary } from "../../shared/TurnChangeSummary.jsx";
import { changes, diffLines } from "../../data/review.js";

export function SourceControlPanel({
  onClose,
  standalone = false,
  clean = false,
  changeSet = changes,
}) {
  const [scope, setScope] = useState("all");
  const [selectedPath, setSelectedPath] = useState(
    () => changeSet[1]?.path ?? changeSet[0]?.path ?? "",
  );
  const [filter, setFilter] = useState("");
  const filtered = (clean ? [] : changeSet).filter((change) => {
    const matchesScope = scope === "all" || (scope === "staged" ? change.staged : change.unstaged);
    return matchesScope && change.path.toLowerCase().includes(filter.toLowerCase());
  });
  const selected = filtered.find((change) => change.path === selectedPath) ?? filtered[0] ?? null;
  return (
    <section className={`workspace-drawer workspace-git ${standalone ? "is-standalone" : ""}`}>
      <header>
        <Icon className="workspace-git-icon" name="git" size={16} />
        <strong>main</strong>
        <span className="workspace-git-divider" aria-hidden="true" />
        <div className="workspace-scope">
          {[
            ["all", "All"],
            ["staged", "Staged"],
            ["unstaged", "Unstaged"],
          ].map(([value, label]) => (
            <button
              key={value}
              type="button"
              className={scope === value ? "is-selected" : ""}
              onClick={() => setScope(value)}
            >
              {label}
            </button>
          ))}
        </div>
        <input
          value={filter}
          onChange={(event) => setFilter(event.target.value)}
          placeholder="Filter changed files"
          aria-label="Filter changed files"
        />
        <IconButton icon="refresh" label="Refresh Git status" onClick={() => setFilter("")} />
        <IconButton
          icon="copy"
          label="Copy patch"
          onClick={() =>
            navigator.clipboard?.writeText(
              diffLines
                .map(
                  (line) =>
                    `${line.kind === "addition" ? "+" : line.kind === "deletion" ? "-" : " "}${line.text}`,
                )
                .join("\n"),
            )
          }
        />
        <IconButton
          icon="close"
          label="Close source control"
          onClick={onClose}
          disabled={!onClose}
        />
      </header>
      <div className="workspace-git-body">
        <div className="workspace-change-list">
          <div className="workspace-drawer-label">
            {filtered.length} {filtered.length === 1 ? "file" : "files"}
          </div>
          {filtered.map((change) => (
            <button
              key={change.path}
              type="button"
              className={selected?.path === change.path ? "is-selected" : ""}
              onClick={() => setSelectedPath(change.path)}
            >
              <b className={`workspace-change-status is-${change.kind}`}>{change.status}</b>
              <span>
                <code>{change.path}</code>
                {change.oldPath && <small>from {change.oldPath}</small>}
              </span>
              <small>
                +{change.additions} -{change.deletions}
              </small>
            </button>
          ))}
          {!filtered.length && !clean && (
            <div className="workspace-change-empty">No changed files match this filter.</div>
          )}
        </div>
        <div className="workspace-diff">
          {selected ? (
            <>
              <div className="workspace-diff-heading">
                <code>{selected.path}</code>
                <span>
                  <b>+{selected.additions}</b> <i>-{selected.deletions}</i>
                </span>
              </div>
              <pre aria-label={`Diff for ${selected.path}`}>
                {diffLines.map((line, index) => (
                  <span
                    key={`${line.kind}-${index}`}
                    className={`workspace-diff-line diff-${line.kind}`}
                  >
                    <span>{line.oldLine}</span>
                    <span>{line.newLine}</span>
                    <i aria-hidden="true" />
                    <code>
                      {line.kind === "addition"
                        ? "+"
                        : line.kind === "deletion"
                          ? "-"
                          : line.kind === "hunk"
                            ? ""
                            : " "}
                      {line.text}
                    </code>
                  </span>
                ))}
              </pre>
            </>
          ) : (
            !clean && (
              <div className="workspace-diff-empty">No changed files match this filter.</div>
            )
          )}
        </div>
      </div>
    </section>
  );
}
