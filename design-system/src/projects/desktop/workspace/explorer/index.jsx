import { useState } from "react";
import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { WorkspaceGuideState } from "../WorkspaceGuide.jsx";
import { ExplorerPanel } from "../WorkspacePrimitives.jsx";

const explorerGuides = {
  withoutDependencies: {
    tabs: {
      actions: ["Expand the project root to browse the active project tree.", "Select a file to focus it and inspect its path.", "Use the dependency root even when it contains no entries."],
      style: ["Rows are 30px high with 3px vertical and 6px horizontal padding.", "Folder and file icons are 14px; indentation advances 12px per tree depth.", "The project tree uses the surface background; selected rows use the active surface."],
      logic: ["The project root is the opened directory and is always the first root.", "A dependency root is present even when no dependency is registered.", "Only read-oriented tree navigation is represented in this surface."],
    },
  },
  withDependencies: {
    tabs: {
      actions: ["Expand Dependencies to inspect each registered read-only root.", "Use the path subtitle on a root row to identify its absolute location.", "Select files in either the project or dependency tree for context."],
      style: ["Dependency rows use a lightly tinted background distinct from project rows.", "Root path subtitles use 9px monospace text and muted contrast.", "Folders, Markdown, code, and configuration files receive different 14px icons."],
      logic: ["Dependencies extend read authority but do not grant write, process, or Git access.", "Each dependency is rendered as a named folder with its registered path.", "The tree keeps project and dependency roots at the same hierarchy level."],
    },
  },
  constrained: {
    tabs: {
      actions: ["Scroll horizontally when a path or nested folder exceeds the panel width.", "Expand deep folders one level at a time to keep the tree readable.", "Select the deeply nested file to verify its full path."],
      style: ["The constrained tree uses a max-content row width with horizontal overflow.", "Long paths are not ellipsized; the monospace path remains fully inspectable.", "The selected file keeps the same 30px row height and active-surface treatment."],
      logic: ["This state represents projects with long absolute paths or deep nesting.", "Horizontal scrolling preserves the canonical path instead of hiding segments.", "Tree depth remains data-driven so additional nesting does not change row geometry."],
    },
  },
};

export function WorkspaceExplorerPage() {
  const [openGuide, setOpenGuide] = useState(null);
  const states = [
    { id: "withoutDependencies", title: "Without dependencies", description: "The project tree is available without registered dependency roots.", side: "right", content: <ExplorerPanel standalone hasDependency={false} /> },
    { id: "withDependencies", title: "With dependencies", description: "Project and read-only dependency roots are shown together.", side: "left", content: <ExplorerPanel standalone hasDependency /> },
    { id: "constrained", title: "Long paths and deep nesting", description: "Full paths remain available when the tree exceeds the panel width.", side: "left", content: <ExplorerPanel standalone hasDependency constrained /> },
  ];
  return <><PageHeader title="Explorer" description="Project files and registered read-only dependency roots within the current project boundary." /><Section id="explorer-panel" title="Project explorer"><div className="workspace-state-grid workspace-explorer-state-grid">{states.map((state) => <WorkspaceGuideState key={state.id} title={state.title} description={state.description} guide={explorerGuides[state.id]} side={state.side} open={openGuide === state.id} onToggle={() => setOpenGuide(openGuide === state.id ? null : state.id)} onClose={() => setOpenGuide(null)}>{state.content}</WorkspaceGuideState>)}</div></Section></>;
}
