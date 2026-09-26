import { SingleDropdown } from "../../../../../components/universal/dropdown/index.js";

import { Icon } from "../../../../../shared/Icon.jsx";

import { ContextUsage } from "../../shared/ContextUsage.jsx";
import { IconButton } from "../../shared/IconButton.jsx";
import { TurnChangeSummary } from "../../shared/TurnChangeSummary.jsx";
import { editorDocuments } from "../../data/editor.js";

const editorCodeLines = [
  [
    ["punctuation", "<"],
    ["keyword", "UserControl"],
    ["punctuation", " "],
    ["type", "xmlns"],
    ["punctuation", '="'],
    ["string", "https://github.com/avaloniaui"],
    ["punctuation", '"'],
    ["punctuation", ">"],
  ],
  [
    ["punctuation", "  <"],
    ["keyword", "Grid"],
    ["punctuation", " "],
    ["type", "RowDefinitions"],
    ["punctuation", '="'],
    ["string", "36,*,20"],
    ["punctuation", '" />'],
  ],
  [["comment", "  <!-- Conversation and editor share this content slot. -->"]],
  [
    ["punctuation", "  <"],
    ["keyword", "Grid"],
    ["punctuation", " "],
    ["type", "Grid.Row"],
    ["punctuation", '="'],
    ["number", "1"],
    ["punctuation", '">'],
  ],
  [
    ["punctuation", "    <"],
    ["keyword", "chat:ChatArea"],
    ["punctuation", " "],
    ["type", "IsVisible"],
    ["punctuation", '="'],
    ["string", "{Binding IsConversationVisible}"],
    ["punctuation", " />"],
  ],
  [
    ["punctuation", "    <"],
    ["keyword", "editor:ReadOnlyEditor"],
    ["punctuation", " "],
    ["type", "IsVisible"],
    ["punctuation", '="'],
    ["string", "{Binding IsEditorVisible}"],
    ["punctuation", " />"],
  ],
  [
    ["punctuation", "  </"],
    ["keyword", "Grid"],
    ["punctuation", ">"],
  ],
  [
    ["punctuation", "  <"],
    ["keyword", "TextBlock"],
    ["punctuation", " "],
    ["type", "Text"],
    ["punctuation", '="'],
    ["string", "READ ONLY"],
    ["punctuation", '" />'],
  ],
  [
    ["punctuation", "</"],
    ["keyword", "UserControl"],
    ["punctuation", ">"],
  ],
];

const markdownEditorLines = [
  [["keyword", "# Shared UI"]],
  [],
  [
    ["punctuation", "This package contains the "],
    ["type", "desktop component foundations"],
    ["punctuation", "."],
  ],
  [],
  [["keyword", "## Usage"]],
  [],
  [
    ["punctuation", "- "],
    ["string", "Import semantic tokens from the shared theme."],
  ],
  [
    ["punctuation", "- "],
    ["string", "Keep project files read-only when opened as dependencies."],
  ],
];

const jsxEditorLines = [
  [
    ["keyword", "export function"],
    ["punctuation", " "],
    ["type", "ModelProviderDropdown"],
    ["punctuation", "({ models }) {"],
  ],
  [
    ["keyword", "  const"],
    ["punctuation", " enabled = models.filter((model) => model.enabled);"],
  ],
  [
    ["keyword", "  return"],
    ["punctuation", " ("],
  ],
  [
    ["punctuation", "    <"],
    ["type", "SingleDropdown"],
    ["punctuation", " options={enabled} ariaLabel="],
    ["string", '"Model provider"'],
    ["punctuation", " />"],
  ],
  [["punctuation", "  );"]],
  [["punctuation", "}"]],
];

export function EditorPanel({
  file,
  state = "ready",
  compact = false,
  standalone = false,
  constrained = false,
}) {
  const document = editorDocuments[file?.id] ?? editorDocuments["workspace-file"];
  const documentLines =
    document.language === "Markdown"
      ? markdownEditorLines
      : document.language === "JavaScript JSX"
        ? jsxEditorLines
        : editorCodeLines;
  const stateLabel =
    state === "loading"
      ? "Loading file"
      : state === "error"
        ? "Unable to read file"
        : state === "empty"
          ? "Empty document"
          : "Read-only document";
  return (
    <section
      className={`workspace-panel workspace-editor ${compact ? "is-compact" : ""} ${standalone ? "is-standalone" : ""} ${constrained ? "is-constrained" : ""} is-${state}`}
      aria-label="Read-only file editor"
    >
      <header className="workspace-editor-header">
        <div className="workspace-editor-file">
          <Icon
            name={
              state === "error"
                ? "file-text"
                : document.language === "AXAML"
                  ? "file-config"
                  : document.language === "Markdown"
                    ? "file-markdown"
                    : "file-code"
            }
            size={16}
          />
          <span>
            <strong>{document.name}</strong>
            <small title={document.path}>{document.path}</small>
          </span>
        </div>
        <div className="workspace-editor-meta">
          <span>{document.language}</span>
          <b>READ ONLY</b>
        </div>
      </header>
      {state === "ready" && (
        <pre
          className="workspace-editor-document"
          aria-label={`${document.name} source`}
          aria-readonly="true"
          tabIndex="0"
        >
          <code>
            {documentLines.map((line, index) => (
              <span className="workspace-editor-line" key={`${document.name}-${index + 1}`}>
                <span className="workspace-editor-line-number">{index + 1}</span>
                <span className="workspace-editor-line-code">
                  {line.map(([tone, text], tokenIndex) => (
                    <span
                      className={`workspace-editor-token is-${tone}`}
                      key={`${index}-${tokenIndex}`}
                    >
                      {text}
                    </span>
                  ))}
                </span>
              </span>
            ))}
          </code>
        </pre>
      )}
      {state !== "ready" && (
        <div
          className={`workspace-editor-state is-${state}`}
          role={state === "error" ? "alert" : "status"}
        >
          <Icon
            name={state === "error" ? "close" : state === "empty" ? "file-text" : "activity"}
            size={24}
          />
          <strong>{stateLabel}</strong>
          <span>
            {state === "loading"
              ? "Reading the selected project file..."
              : state === "error"
                ? "The file could not be read within the project boundary."
                : "This file contains no text to display."}
          </span>
        </div>
      )}
    </section>
  );
}
