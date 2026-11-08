import { useMemo, useRef, useState } from "react";
import { Icon } from "../../../../shared/Icon.jsx";
import { FileIcon } from "../../../../components/universal/file-icon/index.js";
import { statusNames } from "./data.js";

function fileTree(files) {
  const nodes = new Map();
  for (const file of files) {
    const parts = file.path.split("/");
    parts.forEach((name, index) => {
      const path = parts.slice(0, index + 1).join("/");
      nodes.set(path, {
        path,
        name,
        depth: index,
        parent: parts.slice(0, index).join("/"),
        file: index === parts.length - 1 ? file : null,
      });
    });
  }
  return [...nodes.values()].sort((a, b) => a.path.localeCompare(b.path, "en"));
}

export function ChangedFileTree({ files, selectedPath, onSelect }) {
  const nodes = useMemo(() => fileTree(files), [files]);
  const [collapsed, setCollapsed] = useState(new Set());
  const [focusedPath, setFocusedPath] = useState(selectedPath);
  const tree = useRef(null);
  const visible = nodes.filter(
    (node) =>
      !nodes.some(
        (parent) =>
          !parent.file && collapsed.has(parent.path) && node.path.startsWith(`${parent.path}/`),
      ),
  );
  const tabPath = visible.some((node) => node.path === focusedPath)
    ? focusedPath
    : visible[0]?.path;

  function toggle(path, expanded) {
    setCollapsed((current) => {
      const next = new Set(current);
      if (expanded) next.delete(path);
      else next.add(path);
      return next;
    });
  }

  function onKeyDown(event, node) {
    const index = visible.indexOf(node);
    let next;
    switch (event.key) {
      case "ArrowDown":
        next = visible[index + 1];
        break;
      case "ArrowUp":
        next = visible[index - 1];
        break;
      case "Home":
        next = visible[0];
        break;
      case "End":
        next = visible.at(-1);
        break;
      case "ArrowRight":
        if (!node.file && collapsed.has(node.path)) toggle(node.path, true);
        else if (!node.file) next = visible[index + 1];
        break;
      case "ArrowLeft":
        if (!node.file && !collapsed.has(node.path)) toggle(node.path, false);
        else next = visible.find((item) => item.path === node.parent);
        break;
      default:
        return;
    }
    event.preventDefault();
    if (next) {
      setFocusedPath(next.path);
      [...tree.current.querySelectorAll('[role="treeitem"]')]
        .find((item) => item.dataset.path === next.path)
        ?.focus();
    }
  }

  return (
    <div
      ref={tree}
      className="workspace-tree diff-reviewer-tree"
      role="tree"
      aria-label="Changed files"
    >
      {visible.map((node) => (
        <button
          type="button"
          role="treeitem"
          key={node.path}
          data-path={node.path}
          className={`workspace-tree-row ${selectedPath === node.path ? "is-selected" : ""}`}
          style={{ "--tree-depth": node.depth }}
          aria-level={node.depth + 1}
          aria-expanded={node.file ? undefined : !collapsed.has(node.path)}
          aria-selected={selectedPath === node.path}
          aria-label={node.file ? `${node.path}, ${statusNames[node.file.status]}` : node.name}
          tabIndex={tabPath === node.path ? 0 : -1}
          title={node.file?.oldPath ? `${node.path}\nRenamed from ${node.file.oldPath}` : node.path}
          onFocus={() => setFocusedPath(node.path)}
          onKeyDown={(event) => onKeyDown(event, node)}
          onClick={() =>
            node.file ? onSelect(node.path) : toggle(node.path, collapsed.has(node.path))
          }
        >
          {node.file ? (
            <span />
          ) : (
            <Icon
              name="chevron-right"
              size={10}
              className={collapsed.has(node.path) ? "" : "is-open"}
            />
          )}
          {node.file ? (
            <FileIcon type={node.file.icon} size={14} />
          ) : (
            <Icon name="folder" size={14} />
          )}
          <span className="workspace-tree-copy">
            <strong>{node.name}</strong>
          </span>
          {node.file && (
            <span
              className={`diff-reviewer-status is-${node.file.status === "?" ? "untracked" : node.file.status}`}
              aria-hidden="true"
            >
              {node.file.status}
            </span>
          )}
        </button>
      ))}
    </div>
  );
}
