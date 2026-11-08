import { Icon } from "../../../../shared/Icon.jsx";

// The specimen colors illustrative source text. The native design calls for AvaloniaEdit + TextMate.
function Syntax({ text }) {
  const tokens = text.split(
    /("[^"\n]*"|'[^'\n]*'|\/\*.*?\*\/|\b\d+(?:px)?\b|--[\w-]+|\b(?:var|display|grid|color|background|min-width|height|font-family|user-select|public|class)\b)/g,
  );
  return tokens.map((token, index) => {
    let kind = "punctuation";
    if (/^["']/.test(token)) kind = "string";
    else if (token.startsWith("/*")) kind = "comment";
    else if (/^\d/.test(token)) kind = "number";
    else if (token.startsWith("--")) kind = "type";
    else if (/^[a-z][\w-]*$/.test(token)) kind = "keyword";
    return (
      <span key={index} className={`workspace-editor-token is-${kind}`}>
        {token}
      </span>
    );
  });
}

function ChangedText({ text, other }) {
  if (other === undefined || other === text) return <Syntax text={text} />;
  let start = 0;
  while (start < Math.min(text.length, other.length) && text[start] === other[start]) start++;
  let end = 0;
  while (
    end < Math.min(text.length, other.length) - start &&
    text[text.length - end - 1] === other[other.length - end - 1]
  )
    end++;
  return (
    <>
      <Syntax text={text.slice(0, start)} />
      <mark>
        <Syntax text={text.slice(start, text.length - end)} />
      </mark>
      <Syntax text={text.slice(text.length - end)} />
    </>
  );
}

export function DiffDocument({ file, side, expanded, activeChange, onToggleContext }) {
  const otherSide = side === "before" ? "after" : "before";
  return (
    <div className="diff-reviewer-lines">
      {file.blocks.map((block) => {
        const foldable = block.kind === "context" && block.rows.length > 3;
        const isExpanded = expanded.has(block.id);
        return (
          <div
            key={block.id}
            data-change={block.changeIndex ?? undefined}
            className={`diff-reviewer-block ${block.kind === "change" && activeChange === block.changeIndex ? "is-current" : ""}`}
          >
            {foldable && (
              <button
                type="button"
                className="diff-reviewer-fold"
                data-decoration="true"
                aria-expanded={isExpanded}
                onClick={() => onToggleContext(block.id)}
              >
                <Icon name={isExpanded ? "arrow-up" : "expand"} size={12} />
                {isExpanded ? "Collapse" : `${block.rows.length} unmodified lines`}
              </button>
            )}
            {(!foldable || isExpanded) &&
              block.rows.map((row, index) => {
                const line = row[side];
                const other = row[otherSide];
                if (!line)
                  return (
                    <div
                      key={index}
                      className="diff-reviewer-line is-placeholder"
                      aria-hidden="true"
                    />
                  );
                return (
                  <div
                    key={index}
                    className={`diff-reviewer-line ${block.kind === "change" ? `is-${side}` : ""}`}
                  >
                    <span
                      className="diff-reviewer-line-number"
                      data-decoration="true"
                      aria-hidden="true"
                    >
                      {line.number}
                    </span>
                    <code>
                      <ChangedText
                        text={line.text}
                        other={block.kind === "change" ? other?.text : undefined}
                      />
                      {"\n"}
                    </code>
                  </div>
                );
              })}
          </div>
        );
      })}
    </div>
  );
}
