import { useState } from "react";
import { Button } from "../button/index.js";

const diagramSource =
  "flowchart LR\n  A[User submission] --> B[Agent turn]\n  B --> C[Review changes]";
const ganttSource =
  "gantt\n  title Release plan\n  dateFormat YYYY-MM-DD\n  section Build\n  Design :done, design, 2026-11-01, 2d\n  Implement :active, build, after design, 3d\n  Release :milestone, after build, 0d";

// Semantic HTML/MathML and SVG are review specimens, not a Markdown parser.
export function TechnicalMarkdownSpecimen({ CodeBlock }) {
  const [state, setState] = useState("complete");
  const [scale, setScale] = useState(1);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [drag, setDrag] = useState(null);
  const source =
    state === "error"
      ? "unsupportedDiagram\n  A --> B"
      : state === "gantt"
        ? ganttSource
        : diagramSource;

  return (
    <section className="markdown-technical">
      <h2>Mathematics</h2>
      <p>
        Inline notation stays on the reading baseline:{" "}
        <math aria-label="E equals m c squared">
          <mi>E</mi>
          <mo>=</mo>
          <mi>m</mi>
          <msup>
            <mi>c</mi>
            <mn>2</mn>
          </msup>
        </math>
        . The source accepts both dollar and backslash delimiters.
      </p>
      <div className="markdown-math-display" tabIndex={0} aria-label="Display formula">
        <math
          display="block"
          aria-label="x equals negative b plus or minus the square root of b squared minus four a c, divided by two a"
        >
          <mi>x</mi>
          <mo>=</mo>
          <mfrac>
            <mrow>
              <mo>−</mo>
              <mi>b</mi>
              <mo>±</mo>
              <msqrt>
                <msup>
                  <mi>b</mi>
                  <mn>2</mn>
                </msup>
                <mo>−</mo>
                <mn>4</mn>
                <mi>a</mi>
                <mi>c</mi>
              </msqrt>
            </mrow>
            <mrow>
              <mn>2</mn>
              <mi>a</mi>
            </mrow>
          </mfrac>
        </math>
      </div>
      <p className="markdown-secondary">
        Secondary messages preserve their quieter size:{" "}
        <math aria-label="a squared plus b squared">
          <msup>
            <mi>a</mi>
            <mn>2</mn>
          </msup>
          <mo>+</mo>
          <msup>
            <mi>b</mi>
            <mn>2</mn>
          </msup>
        </math>
        .
      </p>
      <div
        className="markdown-math-display"
        tabIndex={0}
        aria-label="Wide formula scrolls horizontally"
      >
        <math display="block" aria-label="A long sum">
          <mi>S</mi>
          <mo>=</mo>
          {Array.from({ length: 24 }, (_, index) => (
            <mrow key={index}>
              {index > 0 && <mo>+</mo>}
              <msub>
                <mi>x</mi>
                <mn>{index + 1}</mn>
              </msub>
            </mrow>
          ))}
        </math>
      </div>
      <p>
        <code className="markdown-math-error" title="Incomplete fraction; source remains readable">
          {"\\frac{1}{"}
        </code>
      </p>
      <p className="markdown-secondary">
        A missing native math library keeps the formula source selectable.
      </p>

      <h2>Mermaid</h2>
      <div className="markdown-diagram-states" role="group" aria-label="Diagram state">
        {["complete", "gantt", "streaming", "error"].map((value) => (
          <Button
            key={value}
            size="sm"
            aria-pressed={state === value}
            onClick={() => setState(value)}
          >
            {value === "complete"
              ? "Complete"
              : value === "gantt"
                ? "Gantt"
                : value === "streaming"
                  ? "Streaming"
                  : "Unsupported"}
          </Button>
        ))}
      </div>
      {state !== "streaming" && (
        <div
          className="markdown-diagram-viewport"
          onWheel={(event) =>
            setScale((value) =>
              Math.min(8, Math.max(0.1, value * (event.deltaY < 0 ? 1.2 : 1 / 1.2))),
            )
          }
          onPointerDown={(event) => {
            event.currentTarget.setPointerCapture(event.pointerId);
            setDrag({ x: event.clientX - offset.x, y: event.clientY - offset.y });
          }}
          onPointerMove={(event) => {
            if (drag) setOffset({ x: event.clientX - drag.x, y: event.clientY - drag.y });
          }}
          onPointerUp={() => setDrag(null)}
          onPointerCancel={() => setDrag(null)}
        >
          {state === "error" ? (
            <p role="status">This diagram type is not supported. The source is available below.</p>
          ) : state === "gantt" ? (
            <svg
              viewBox="0 0 680 260"
              role="img"
              aria-label="Release plan: design November 1 to 3, implement November 3 to 6, release November 6"
              style={{ transform: `translate(${offset.x}px, ${offset.y}px) scale(${scale})` }}
            >
              <text x="16" y="28" className="markdown-gantt-title">
                Release plan
              </text>
              <rect className="markdown-gantt-done" x="426" y="17" width="10" height="10" rx="3" />
              <text x="443" y="26">
                Done
              </text>
              <rect
                className="markdown-gantt-active"
                x="492"
                y="17"
                width="10"
                height="10"
                rx="3"
              />
              <text x="509" y="26">
                Active
              </text>
              <rect
                className="markdown-gantt-milestone"
                x="572"
                y="17"
                width="10"
                height="10"
                rx="3"
              />
              <text x="589" y="26">
                Planned
              </text>
              <rect
                className="markdown-gantt-panel"
                x="8"
                y="50"
                width="664"
                height="180"
                rx="10"
              />
              <path
                className="markdown-diagram-edge"
                d="M160 60V230M260 60V230M360 60V230M460 60V230M560 60V230M650 60V230"
              />
              {[
                [160, "Nov 1"],
                [360, "Nov 3"],
                [650, "Nov 6"],
              ].map(([x, label]) => (
                <text key={label} x={x} y="250" textAnchor="middle">
                  {label}
                </text>
              ))}
              <text x="16" y="90">
                Build
              </text>
              <text x="16" y="125">
                Design
              </text>
              <rect
                className="markdown-gantt-done"
                x="160"
                y="108"
                width="200"
                height="24"
                rx="4"
              />
              <text x="16" y="169">
                Implement
              </text>
              <rect
                className="markdown-gantt-active"
                x="360"
                y="152"
                width="290"
                height="24"
                rx="4"
              />
              <text x="16" y="213">
                Release
              </text>
              <path className="markdown-gantt-milestone" d="M650 198l8 8-8 8-8-8Z" />
            </svg>
          ) : (
            <svg
              viewBox="0 0 680 140"
              role="img"
              aria-label="User submission leads to agent turn, then review changes"
              style={{ transform: `translate(${offset.x}px, ${offset.y}px) scale(${scale})` }}
            >
              <path
                className="markdown-diagram-edge"
                d="M190 70H250m-8-5 8 5-8 5M430 70H490m-8-5 8 5-8 5"
              />
              {[
                [10, "User submission"],
                [250, "Agent turn"],
                [490, "Review changes"],
              ].map(([x, label]) => (
                <g key={label}>
                  <rect x={x} y="42" width="180" height="56" rx="6" />
                  <text x={x + 90} y="75" textAnchor="middle">
                    {label}
                  </text>
                </g>
              ))}
            </svg>
          )}
        </div>
      )}
      <details key={state} className="markdown-diagram-source" open={state === "streaming"}>
        <summary>Mermaid source</summary>
        <CodeBlock language="mermaid">{source}</CodeBlock>
      </details>
    </section>
  );
}
