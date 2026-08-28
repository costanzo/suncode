import specimenImage from "../../../assets/logos/suncode-logo.svg";
import "./Markdown.css";

const javascriptExample = `function createSession(projectId) {
  return client.sessions.create({
    projectId,
    model: "gpt-5.6-sol",
  });
}`;

const shellExample = `suncode project open ./sample-project
suncode session create --model gpt-5.6-sol`;

export function MarkdownSpecimen() {
  return (
    <article className="markdown-sample">
      <div className="markdown">
        <h1>Building a reliable coding session</h1>
        <p>
          A good session keeps the project context clear, makes operations reviewable,
          and leaves the user in control of every sensitive action.
        </p>

        <h2>Text hierarchy</h2>
        <p>
          Markdown prose supports <strong>strong emphasis</strong>, <em>subtle emphasis</em>,
          {" "}<strong><em>combined emphasis</em></strong>, <del>superseded guidance</del>,
          {" "}<u>underlined text</u>, H<sub>2</sub>O, and x<sup>2</sup>. Use
          {" "}<code>inline code</code> for commands, paths, identifiers, and short values.
        </p>
        <p>
          Links remain visible without depending on color alone. Read the
          {" "}<a href="https://commonmark.org" target="_blank" rel="noreferrer">CommonMark specification</a>
          {" "}or jump to the <a href="#markdown-tables">table example</a> below.
        </p>

        <h2>Links</h2>
        <p>
          A named link points to a related destination: <a href="https://github.com" target="_blank" rel="noreferrer">Visit GitHub</a>.
        </p>
        <ul>
          <li><a href="https://commonmark.org" target="_blank" rel="noreferrer">External link</a> opens a separate destination.</li>
          <li><a href="#markdown-tables">Anchor link</a> moves within the same document.</li>
          <li><a href="https://www.google.com" target="_blank" rel="noreferrer">https://www.google.com</a> shows an automatic URL.</li>
          <li><a href="mailto:hello@example.com">hello@example.com</a> uses a mail link.</li>
        </ul>

        <h3>Third-level heading</h3>
        <p>Use this level to separate closely related topics inside a larger section.</p>
        <h4>Fourth-level heading</h4>
        <p>Deeper headings stay distinct without competing with the document title.</p>
        <h5>Fifth-level heading</h5>
        <p>This level works best for compact reference material.</p>
        <h6>Sixth-level heading</h6>
        <p>The smallest heading is still recognizable as structure, not metadata.</p>

        <hr />

        <h2>Lists</h2>
        <h3>Unordered and nested lists</h3>
        <ul>
          <li>Open a local project.</li>
          <li>
            Create a session with the right provider.
            <ul>
              <li>Confirm the selected model.</li>
              <li>
                Review project authority.
                <ul>
                  <li>Keep external paths read-only unless explicitly approved.</li>
                </ul>
              </li>
            </ul>
          </li>
          <li>Submit the first turn.</li>
        </ul>

        <h3>Ordered lists</h3>
        <ol>
          <li>Inspect the affected files.</li>
          <li>
            Make the smallest coherent change.
            <ol>
              <li>Verify focused behavior.</li>
              <li>Inspect the final diff.</li>
            </ol>
          </li>
          <li>Report the result and any remaining limitation.</li>
        </ol>

        <h3>Task lists</h3>
        <ul className="contains-task-list">
          <li className="task-list-item"><input type="checkbox" checked readOnly />Project opened</li>
          <li className="task-list-item"><input type="checkbox" checked readOnly />Focused checks passed</li>
          <li className="task-list-item"><input type="checkbox" readOnly />Broader integration check pending</li>
        </ul>

        <hr />

        <h2>Blockquotes</h2>
        <blockquote>
          <p>Authority should be understandable before an operation runs, not reconstructed afterward.</p>
        </blockquote>
        <blockquote>
          <p>A blockquote can contain richer Markdown structures.</p>
          <ul>
            <li>Lists remain readable.</li>
            <li>Inline <code>machine values</code> keep their semantic treatment.</li>
          </ul>
          <blockquote>
            <p>Nested quotes use quieter tonal separation.</p>
          </blockquote>
        </blockquote>

        <h2>Code</h2>
        <p>Inline code such as <code>session.create</code> sits naturally within prose.</p>
        <pre><code>{javascriptExample}</code></pre>
        <pre><code>{shellExample}</code></pre>

        <h2>Image</h2>
        <p>Images respect the reading measure and never overflow their container.</p>
        <img src={specimenImage} alt="SunCode monochrome application mark" />

        <h2 id="markdown-tables">Tables</h2>
        <div className="markdown-table-wrap">
          <table className="markdown-table">
            <thead>
              <tr>
                <th scope="col">Element</th>
                <th scope="col">Purpose</th>
                <th scope="col">Support</th>
              </tr>
            </thead>
            <tbody>
              <tr><td><strong>Heading</strong></td><td>Document hierarchy</td><td>Supported</td></tr>
              <tr><td><code>Code</code></td><td>Commands and identifiers</td><td>Supported</td></tr>
              <tr><td><a href="#markdown-tables">Link</a></td><td>Related destinations</td><td>Supported</td></tr>
              <tr><td><del>Deleted text</del></td><td>Superseded content</td><td>Supported</td></tr>
            </tbody>
          </table>
        </div>

        <h2>Extended inline elements</h2>
        <p>
          Press <kbd>Command</kbd> + <kbd>Enter</kbd> to submit. Use
          {" "}<mark>highlighted text</mark> only when the content itself calls for emphasis.
        </p>

        <hr />

        <section className="markdown-footnotes" aria-label="Footnotes">
          <h2>Footnotes</h2>
          <p>
            Durable state belongs to the Rust-owned data layer.
            <sup><a href="#markdown-footnote-1" id="markdown-footnote-ref-1">1</a></sup>
          </p>
          <ol>
            <li id="markdown-footnote-1">
              Clients consume the SDK contract and do not access SQLite directly.
              {" "}<a className="markdown-footnote-backref" href="#markdown-footnote-ref-1" aria-label="Back to footnote reference">Back</a>
            </li>
          </ol>
        </section>
      </div>
    </article>
  );
}
