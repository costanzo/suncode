import { useState } from "react";
import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { WorkspaceGuideState } from "../WorkspaceGuide.jsx";
import { EditorPanel, ExplorerPanel } from "../WorkspacePrimitives.jsx";

const editorGuides = {
  ready: {
    tabs: {
      actions: [
        "Select a file in Explorer to replace Conversation with its read-only document view.",
        "Select a session in the Sessions list to return to that session's conversation.",
        "Select and copy text when a snippet is needed as context; no edit action is available.",
      ],
      style: [
        "The editor header is 44px high and shows the file identity, language, and READ ONLY status.",
        "Line numbers use the mono data font and a quiet gutter; syntax roles remain legible in both themes.",
        "The document surface keeps a stable content width and scrolls long lines horizontally.",
      ],
      logic: [
        "The selected file is transient Workspace state and is not a new session.",
        "AvaloniaEdit renders the document read-only while TextMate supplies language-aware highlighting.",
        "The selected session remains the return point for conversation history.",
      ],
    },
  },
  loading: {
    tabs: {
      actions: ["Wait for the bounded file read to finish before inspecting the document."],
      style: [
        "Keep the file identity and READ ONLY status visible while the body reports progress.",
        "Use the existing activity icon and muted status hierarchy; do not show an editable placeholder.",
      ],
      logic: [
        "The editor is waiting for the client API to return the selected file contents.",
        "A loading view never implies that a file is available or editable.",
      ],
    },
  },
  error: {
    tabs: {
      actions: [
        "Return to Sessions to continue the conversation or select another file in Explorer.",
      ],
      style: [
        "Keep the error bounded to the editor body and preserve the file path in the header.",
        "Use the danger token only for the actual read failure state.",
      ],
      logic: [
        "The read request failed or the file moved outside the project boundary.",
        "No retry or mutation control is implied by this read-only design specimen.",
      ],
    },
  },
};

function EditorComposition({ initialState = "ready", constrained = false }) {
  const [state, setState] = useState(initialState);
  const [selectedFile, setSelectedFile] = useState(
    constrained
      ? {
          id: "stress-dropdown",
          name: "ModelProviderDropdown.jsx",
          path: "frontend/components/selection/ModelProviderDropdown.jsx",
        }
      : null,
  );
  return (
    <div className={`workspace-editor-composition ${constrained ? "is-constrained" : ""}`}>
      <ExplorerPanel
        standalone
        constrained={constrained}
        selectedFileId={selectedFile?.id}
        onFileSelect={(file) => {
          setSelectedFile(file);
          setState("ready");
        }}
      />
      <div className="workspace-editor-composition-main">
        <div className="workspace-editor-mode-row">
          <span>EDITOR MODE</span>
          <div role="group" aria-label="Editor specimen state">
            {[
              ["ready", "Ready"],
              ["loading", "Loading"],
              ["empty", "Empty"],
              ["error", "Read failure"],
            ].map(([value, label]) => (
              <button
                key={value}
                type="button"
                className={state === value ? "is-selected" : ""}
                onClick={() => setState(value)}
              >
                {label}
              </button>
            ))}
          </div>
        </div>
        <EditorPanel standalone constrained={constrained} file={selectedFile} state={state} />
      </div>
    </div>
  );
}

export function WorkspaceEditorPage() {
  const [openGuide, setOpenGuide] = useState(null);
  const states = [
    {
      id: "ready",
      title: "File selected",
      description:
        "Explorer selection replaces the central conversation with a read-only document.",
      side: "right",
      content: <EditorComposition />,
    },
    {
      id: "loading",
      title: "Loading file",
      description: "The file identity remains visible while a bounded read is in progress.",
      side: "left",
      content: <EditorComposition initialState="loading" />,
    },
    {
      id: "error",
      title: "Read failure",
      description: "A failed read stays explicit without adding edit or retry affordances.",
      side: "right",
      content: <EditorComposition initialState="error" />,
    },
    {
      id: "constrained",
      title: "Constrained width",
      description: "Deep paths and long lines remain inspectable through horizontal overflow.",
      side: "left",
      content: <EditorComposition constrained />,
    },
  ];
  return (
    <>
      <PageHeader
        title="Editor"
        description="Read-only project file viewing inside the Workspace conversation region, with TextMate-style syntax highlighting."
      />
      <Section
        id="editor-panel"
        title="Read-only editor"
        description="Select a file in Explorer to inspect it; select a session to return to Conversation."
      >
        <div className="workspace-editor-state-grid">
          {states.map((state) => {
            const guide = editorGuides[state.id === "constrained" ? "ready" : state.id];
            const guideOpen = openGuide === state.id;
            return (
              <WorkspaceGuideState
                key={state.id}
                className="workspace-editor-specimen"
                title={state.title}
                description={state.description}
                guide={guide}
                side={state.side}
                open={guideOpen}
                onToggle={() => setOpenGuide(guideOpen ? null : state.id)}
                onClose={() => setOpenGuide(null)}
              >
                {state.content}
              </WorkspaceGuideState>
            );
          })}
        </div>
      </Section>
    </>
  );
}
