import { Icon } from "../../../shared/Icon.jsx";
import { Specimen } from "../Specimen.jsx";
import { FileIcon, getFileIconDefinition } from "./FileIcon.jsx";

const iconGroups = [
  {
    label: "Language and markup",
    entries: [
      ["javascript", "JavaScript", ".js .mjs .cjs"],
      ["typescript", "TypeScript", ".ts"],
      ["react", "React", ".jsx .tsx"],
      ["rust", "Rust", ".rs"],
      ["python", "Python", ".py"],
      ["csharp", "C Sharp", ".cs"],
      ["html", "HTML", ".html .axaml"],
      ["css", "CSS", ".css .scss"],
      ["markdown", "Markdown", ".md"],
    ],
  },
  {
    label: "Configuration and project",
    entries: [
      ["json", "JSON", ".json .jsonc"],
      ["yaml", "YAML", ".yaml .yml"],
      ["config", "Configuration", ".toml .env"],
      ["shell", "Shell", ".sh"],
      ["docker", "Docker", "Dockerfile"],
      ["git", "Git", ".gitignore"],
      ["database", "Database", ".sql"],
      ["license", "License", "LICENSE"],
      ["todo", "Todo", "TODO.md"],
      ["info", "Information", "README.md"],
      ["tsconfig", "TypeScript config", "tsconfig.json"],
    ],
  },
  {
    label: "Documents and assets",
    entries: [
      ["image", "Image", ".png .jpg .webp"],
      ["svg", "SVG", ".svg"],
      ["audio", "Audio", ".mp3 .wav"],
      ["video", "Video", ".mp4 .webm"],
      ["pdf", "PDF", ".pdf"],
      ["archive", "Archive", ".zip .jar"],
      ["default", "Generic", "unrecognized"],
    ],
  },
];

const explorerRows = [
  { kind: "folder", name: "apps", detail: "folder" },
  { kind: "folder", name: "desktop-avalonia", detail: "folder" },
  { type: "csharp", name: "ProjectWorkspaceViewModel.cs", detail: "C Sharp" },
  { type: "info", name: "README.md", detail: "filename match" },
  { type: "tsconfig", name: "tsconfig.json", detail: "filename match" },
  { type: "config", name: "Cargo.toml", detail: "TOML" },
  { type: "git", name: ".gitignore", detail: "Git" },
];

export function FileIconSpecimens() {
  return (
    <div className="file-icon-specimens">
      <Specimen label="Explorer row application">
        <div className="file-icon-explorer" role="tree" aria-label="File icon explorer example">
          {explorerRows.map((row, index) => (
            <button
              key={row.name}
              className={`file-icon-explorer-row ${index === 3 ? "is-selected" : ""}`}
              type="button"
              role="treeitem"
              aria-level={row.kind === "folder" ? index + 1 : 3}
              aria-selected={index === 3}
            >
              <span className="file-icon-explorer-indent" aria-hidden="true" />
              {row.kind === "folder" ? (
                <Icon name="folder" size={14} />
              ) : (
                <FileIcon type={row.type} size={14} />
              )}
              <span className="file-icon-explorer-name">{row.name}</span>
              <span className="file-icon-explorer-detail">{row.detail}</span>
            </button>
          ))}
        </div>
        <p className="sample-note">
          File glyphs retain their type color in rest, hover, focus, and selected rows. Folder
          glyphs remain part of the monochrome interface icon family.
        </p>
      </Specimen>

      <div className="file-icon-catalog">
        {iconGroups.map((group) => (
          <Specimen key={group.label} label={group.label}>
            <div className="file-icon-grid">
              {group.entries.map(([type, name, associations]) => {
                const definition = getFileIconDefinition(type);
                return (
                  <div className="file-icon-cell" key={type}>
                    <FileIcon type={type} size={20} />
                    <div>
                      <strong>{name}</strong>
                      <code>{associations}</code>
                    </div>
                    <code className="file-icon-codepoint">
                      {definition.glyph.codePointAt(0).toString(16)}
                    </code>
                  </div>
                );
              })}
            </div>
          </Specimen>
        ))}
      </div>
    </div>
  );
}
