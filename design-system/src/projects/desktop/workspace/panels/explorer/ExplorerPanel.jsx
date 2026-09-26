import { Icon } from "../../../../../shared/Icon.jsx";

import {
  constrainedDependencyRoot,
  constrainedExplorerNodes,
  constrainedProjectRoot,
  dependencyNodes,
  dependencyRoot,
  explorerNodes,
  projectRoot,
} from "../../data/explorer.js";
import { IconButton } from "../../shared/IconButton.jsx";

export function ExplorerPanel({
  compact = false,
  standalone = false,
  hasDependency = true,
  constrained = false,
  onFileSelect,
  selectedFileId,
}) {
  const roots = constrained
    ? [constrainedProjectRoot, constrainedDependencyRoot]
    : [projectRoot, dependencyRoot];
  const treeNodes = constrained ? constrainedExplorerNodes : explorerNodes;
  const dependencyTreeNodes = constrained
    ? constrainedExplorerNodes.filter((node) => node.isDependency)
    : dependencyNodes;
  const nodes = [
    ...roots.slice(0, 1),
    ...treeNodes.filter((node) => !node.isDependency),
    roots[1],
    ...(hasDependency ? dependencyTreeNodes : []),
  ];
  const [expanded, setExpanded] = useState(
    () =>
      new Set(
        constrained
          ? [
              "stress-project-root",
              "stress-agents",
              "stress-requirements",
              "stress-package",
              "stress-frontend",
              "stress-components",
              "stress-selection",
              "stress-dependencies",
            ]
          : ["project-root", "apps", "desktop", "views", "dependencies", "shared"],
      ),
  );
  const [localSelectedFileId, setLocalSelectedFileId] = useState(
    () => nodes.find((node) => node.kind === "file" && node.selected)?.id ?? null,
  );
  const activeFileId = selectedFileId ?? localSelectedFileId;
  const visibleNodes = nodes.filter((node) => {
    let parentId = node.parent;
    while (parentId) {
      if (!expanded.has(parentId)) return false;
      parentId = nodes.find((candidate) => candidate.id === parentId)?.parent;
    }
    return true;
  });
  const toggleNode = (node) => {
    if (node.kind === "file") return;
    setExpanded((current) => {
      const next = new Set(current);
      if (next.has(node.id)) next.delete(node.id);
      else next.add(node.id);
      return next;
    });
  };
  return (
    <aside
      className={`workspace-panel workspace-explorer ${compact ? "is-compact" : ""} ${standalone ? "is-standalone" : ""} ${constrained ? "is-constrained" : ""}`}
    >
      <header className="workspace-panel-header">
        <span>EXPLORER</span>
        <span className="workspace-panel-actions">
          <IconButton icon="refresh" label="Refresh explorer" disabled />
          <IconButton icon="plus" label="Add dependency folder" disabled />
        </span>
      </header>
      <div className="workspace-tree" role="tree" aria-label="Project files">
        {visibleNodes.map((node) => {
          const isContainer = node.kind !== "file";
          const dependencyParent = constrained ? "stress-dependencies" : "dependencies";
          const isDependencyNode = node.isDependency || node.parent === dependencyParent;
          const extension = node.name.includes(".") ? node.name.split(".").pop().toLowerCase() : "";
          const fileIcon =
            extension === "md"
              ? "file-markdown"
              : ["jsx", "js", "ts", "tsx", "rs", "go", "java", "py"].includes(extension)
                ? "file-code"
                : ["json", "yaml", "yml", "toml", "xml", "axaml"].includes(extension)
                  ? "file-config"
                  : extension
                    ? "file-text"
                    : "file";
          const showPath = node.parent === dependencyParent;
          const iconName =
            node.kind === "project-root"
              ? "project"
              : node.kind === "dependencies"
                ? "dependencies"
                : isContainer
                  ? "folder"
                  : fileIcon;
          return (
            <button
              key={node.id}
              type="button"
              role="treeitem"
              aria-level={node.depth + 1}
              aria-expanded={isContainer ? expanded.has(node.id) : undefined}
              aria-selected={activeFileId === node.id || undefined}
              className={`workspace-tree-row ${activeFileId === node.id ? "is-selected" : ""} ${isDependencyNode ? "is-dependency" : ""} ${node.kind === "dependencies" ? "is-dependency-root" : ""} ${node.kind === "workspace" ? "is-workspace-root" : ""}`}
              style={{ "--tree-depth": node.depth }}
              onClick={() => {
                if (isContainer) {
                  toggleNode(node);
                  return;
                }
                setLocalSelectedFileId(node.id);
                onFileSelect?.(node);
              }}
            >
              {isContainer ? (
                <Icon
                  name="chevron-right"
                  className={expanded.has(node.id) ? "is-open" : ""}
                  size={8}
                />
              ) : (
                <span />
              )}
              <Icon name={iconName} size={14} />
              <span className="workspace-tree-copy" title={node.path || node.name}>
                <strong>{node.name}</strong>
                {showPath && node.path && <small>{node.path}</small>}
              </span>
            </button>
          );
        })}
      </div>
    </aside>
  );
}
