import { useState } from "react";
import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { WorkspaceGuideState } from "../WorkspaceGuide.jsx";
import { SourceControlPanel } from "../WorkspacePrimitives.jsx";

const sourceControlGuides = {
  clean: {
    tabs: {
      actions: [
        "Confirm the repository has no changed files.",
        "Use the workspace conversation to start a turn that can produce a diff.",
        "Return here after a turn to verify its source changes.",
      ],
      style: [
        "The clean state uses a centered empty message inside the diff area.",
        "Repository chrome stays at the same 36px header height as the changed state.",
        "No green clean-status banner is shown; the absence of rows is the signal.",
      ],
      logic: [
        "The read-only Git status contains no staged or unstaged changes.",
        "No diff can be selected while the repository is clean.",
        "The panel never mutates Git state; it only presents the current snapshot.",
      ],
    },
  },
  changed: {
    tabs: {
      actions: [
        "Switch between All, Staged, and Unstaged scopes.",
        "Select a changed file to inspect its line-level diff.",
        "Use the filter, refresh, or copy patch controls from the header.",
      ],
      style: [
        "The drawer uses a 36px toolbar and a 230px file list at desktop widths.",
        "Diff rows are 22px high with 34px old/new line-number columns.",
        "Added, deleted, and modified markers use success, danger, and secondary tones.",
      ],
      logic: [
        "The list is derived from the current repository snapshot.",
        "Filtering changes the selected file and diff without changing the repository.",
        "Turn summaries can open this view with the turn's complete change set selected.",
      ],
    },
  },
};

export function WorkspaceSourceControlPage() {
  const [openGuide, setOpenGuide] = useState(null);
  const states = [
    {
      id: "clean",
      title: "No changes",
      description: "The repository snapshot has no staged or unstaged files.",
      side: "right",
      content: <SourceControlPanel standalone clean />,
    },
    {
      id: "changed",
      title: "Changed files",
      description: "Changed files and their line-level patch are available for review.",
      side: "left",
      content: <SourceControlPanel standalone />,
    },
  ];
  return (
    <>
      <PageHeader
        title="Source control"
        description="Read-only repository status and patch review for the open project."
      />
      <Section id="source-control-panel" title="Changed files and diff">
        <div className="workspace-state-grid workspace-drawer-state-grid">
          {states.map((state) => (
            <WorkspaceGuideState
              key={state.id}
              title={state.title}
              description={state.description}
              guide={sourceControlGuides[state.id]}
              side={state.side}
              open={openGuide === state.id}
              onToggle={() => setOpenGuide(openGuide === state.id ? null : state.id)}
              onClose={() => setOpenGuide(null)}
            >
              {state.content}
            </WorkspaceGuideState>
          ))}
        </div>
      </Section>
    </>
  );
}
