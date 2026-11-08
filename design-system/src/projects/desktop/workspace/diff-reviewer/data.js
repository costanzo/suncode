// Illustrative Git comparisons for the design review surface.
const context = (lines) => ({ kind: "context", lines });
const change = (before, after) => ({ kind: "change", before, after });

function comparison(path, status, icon, scopes, sections, extra = {}) {
  let oldLine = 1;
  let newLine = 1;
  let changes = 0;
  let additions = 0;
  let deletions = 0;
  const merged = [];
  for (const section of sections) {
    const previous = merged.at(-1);
    if (section.kind === "context" && previous?.kind === "context") {
      previous.lines.push(...section.lines);
    } else {
      merged.push(section.kind === "context" ? context([...section.lines]) : section);
    }
  }
  const blocks = merged.map((section, index) => {
    const before = section.kind === "context" ? section.lines : section.before;
    const after = section.kind === "context" ? section.lines : section.after;
    const changeIndex = section.kind === "change" ? changes++ : null;
    if (section.kind === "change") {
      additions += after.length;
      deletions += before.length;
    }
    const rows = Array.from({ length: Math.max(before.length, after.length) }, (_, row) => ({
      before: before[row] === undefined ? null : { number: oldLine++, text: before[row] },
      after: after[row] === undefined ? null : { number: newLine++, text: after[row] },
    }));
    return { id: `block-${index}`, kind: section.kind, changeIndex, rows };
  });
  return { path, status, icon, scopes, blocks, changes, additions, deletions, ...extra };
}

const layoutRules = [
  "header",
  "body",
  "sidebar",
  "tree",
  "row",
  "detail",
  "toolbar",
  "path",
  "stats",
  "gutter",
  "code",
  "context",
  "footer",
  "actions",
];
const unchangedRules = layoutRules.flatMap((name) => [
  `.review-${name} {`,
  "  min-width: 0;",
  "  color: var(--text-secondary);",
  "}",
  "",
]);

export const comparisons = [
  comparison(
    "design-system/src/projects/desktop/workspace/styles/review-diff.css",
    "M",
    "css",
    ["staged", "unstaged"],
    [
      context([
        "/* Shared change review surfaces. */",
        "",
        ".workspace-diff {",
        "  min-width: 0;",
        "  background: var(--surface);",
        "}",
        "",
        "/* File comparison */",
      ]),
      change([".workspace-diff {"], [".workspace-diff,", ".diff-reviewer-document {"]),
      context(["  font-family: var(--font-mono);", "  white-space: pre;", "}", ""]),
      change(
        [".workspace-diff-line { height: 20px; }"],
        [".workspace-diff-line { height: 22px; }"],
      ),
      context(unchangedRules),
      change(
        [],
        [
          ".diff-reviewer-body {",
          "  display: grid;",
          "  grid-template-columns: var(--diff-reviewer-tree-width) minmax(0, 1fr);",
          "}",
          "",
          ".diff-reviewer-placeholder {",
          "  user-select: none;",
          "}",
        ],
      ),
      context(["", ".workspace-diff code {", "  tab-size: 2;", "}"]),
    ],
  ),
  comparison(
    "apps/desktop-avalonia/Views/Review/DiffReviewerWindow.axaml",
    "A",
    "config",
    ["staged"],
    [
      change(
        [],
        [
          '<Window xmlns="https://github.com/avaloniaui"',
          '        Title="Diff reviewer"',
          '        WindowDecorations="Full">',
          '  <Grid ColumnDefinitions="240,1,*">',
          '    <TreeView ItemsSource="{Binding ChangedFiles}" />',
          '    <GridSplitter Grid.Column="1" />',
          '    <ContentControl Grid.Column="2" Content="{Binding Comparison}" />',
          "  </Grid>",
          "</Window>",
        ],
      ),
    ],
  ),
  comparison(
    "apps/desktop-avalonia/Views/Review/GitViewer.axaml",
    "M",
    "config",
    ["unstaged"],
    [
      context([
        '<UserControl xmlns="https://github.com/avaloniaui">',
        "  <Grid>",
        '    <StackPanel Orientation="Horizontal">',
      ]),
      change(
        ['      <Button Content="Refresh" />'],
        [
          '      <Button Content="Refresh" />',
          '      <Button Content="Open diff reviewer"',
          '              Command="{Binding OpenDiffReviewer}" />',
        ],
      ),
      context(["    </StackPanel>", "  </Grid>", "</UserControl>"]),
    ],
  ),
  comparison(
    "design-system/src/styles/tokens/diff-reviewer.css",
    "?",
    "css",
    ["unstaged"],
    [
      change(
        [],
        [
          ":root {",
          "  --diff-reviewer-tree-width: 240px;",
          "  --diff-reviewer-row-height: 22px;",
          "  --diff-reviewer-gutter-width: 42px;",
          "}",
        ],
      ),
    ],
  ),
  comparison(
    "docs/diff-review.md",
    "R",
    "markdown",
    ["staged"],
    [
      context(["# Reviewing changes", ""]),
      change(
        ["Read the patch in Source control."],
        [
          "Open Diff reviewer from Source control.",
          "Select a changed file in the tree.",
          "Use Previous difference and Next difference to review each block.",
        ],
      ),
      context(["", "The selected Git range determines the compared versions."]),
    ],
    { oldPath: "docs/patch-review.md" },
  ),
  comparison(
    "docs/legacy-review.md",
    "D",
    "markdown",
    ["staged"],
    [
      change(
        [
          "# Legacy review",
          "",
          "Open a patch and inspect its changed lines.",
          "This guide has moved to diff-review.md.",
        ],
        [],
      ),
    ],
  ),
  comparison("assets/review-preview.png", "M", "image", ["unstaged"], [], { binary: true }),
];

// The first edit is staged. The other two are worktree-only, so both Index views agree.
function cssComparisonForRange(scope) {
  const file = comparisons[0];
  const sections = file.blocks.map((block) => {
    const before = block.rows.filter((row) => row.before).map((row) => row.before.text);
    const after = block.rows.filter((row) => row.after).map((row) => row.after.text);
    if (block.kind === "context") return context(before);
    const staged = block.changeIndex === 0;
    if ((scope === "staged") === staged) return change(before, after);
    return context(scope === "staged" ? before : after);
  });
  return comparison(file.path, file.status, file.icon, [scope], sections);
}

export const comparisonsByRange = {
  all: comparisons,
  staged: comparisons
    .filter((file) => file.scopes.includes("staged"))
    .map((file) => (file === comparisons[0] ? cssComparisonForRange("staged") : file)),
  unstaged: comparisons
    .filter((file) => file.scopes.includes("unstaged"))
    .map((file) => (file === comparisons[0] ? cssComparisonForRange("unstaged") : file)),
};

export const statusNames = {
  M: "Modified",
  A: "Added",
  D: "Deleted",
  R: "Renamed",
  "?": "Untracked",
};

export const comparisonSides = {
  all: ["HEAD", "Working tree"],
  staged: ["HEAD", "Index"],
  unstaged: ["Index", "Working tree"],
};
