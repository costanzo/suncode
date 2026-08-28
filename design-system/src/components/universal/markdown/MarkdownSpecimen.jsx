import { useEffect, useRef, useState } from "react";
import specimenImage from "../../../assets/logos/suncode-logo.svg";
import { Icon } from "../../../shared/Icon.jsx";
import "./Markdown.css";

function CodeBlock({ language, children }) {
  const codeRef = useRef(null);
  const resetTimerRef = useRef(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => () => window.clearTimeout(resetTimerRef.current), []);

  async function copyCode() {
    const code = codeRef.current;
    if (!code) return;

    try {
      if (navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(code.textContent ?? "");
      } else {
        const selection = window.getSelection();
        const range = document.createRange();
        range.selectNodeContents(code);
        selection?.removeAllRanges();
        selection?.addRange(range);
        document.execCommand("copy");
        selection?.removeAllRanges();
      }

      setCopied(true);
      window.clearTimeout(resetTimerRef.current);
      resetTimerRef.current = window.setTimeout(() => setCopied(false), 1600);
    } catch {
      setCopied(false);
    }
  }

  return (
    <div className="markdown-code-block">
      <button
        type="button"
        className={`markdown-copy-button${copied ? " is-copied" : ""}`}
        aria-label={copied ? `${language} code copied` : `Copy ${language} code`}
        title={copied ? "Copied" : "Copy code"}
        onClick={copyCode}
      >
        <Icon name={copied ? "check" : "copy"} size={14} />
      </button>
      <pre><code ref={codeRef} className={`language-${language}`}>{children}</code></pre>
    </div>
  );
}

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

        <h3>JavaScript</h3>
        <CodeBlock language="javascript">
          <span className="token-comment">// Create a session for the active project</span>{"\n"}
          <span className="token-keyword">async function</span>{" "}<span className="token-function">createSession</span>(<span className="token-parameter">projectId</span>) {"{"}{"\n"}
          {"  "}<span className="token-keyword">const</span> session = <span className="token-keyword">await</span> client.sessions.<span className="token-function">create</span>({"{"}{"\n"}
          {"    "}projectId,{"\n"}
          {"    "}model: <span className="token-string">"gpt-5.6-sol"</span>,{"\n"}
          {"    "}stream: <span className="token-boolean">true</span>,{"\n"}
          {"  "}{"}"});{"\n"}
          {"  "}<span className="token-keyword">return</span> session.id;{"\n"}
          {"}"}
        </CodeBlock>

        <h3>Java</h3>
        <CodeBlock language="java">
          <span className="token-annotation">@Service</span>{"\n"}
          <span className="token-keyword">public final class</span>{" "}<span className="token-type">SessionService</span>{" "}{"{"}{"\n"}
          {"  "}<span className="token-keyword">public</span>{" "}<span className="token-type">Session</span>{" "}<span className="token-function">create</span>(<span className="token-type">String</span> projectId) {"{"}{"\n"}
          {"    "}<span className="token-keyword">var</span> model = <span className="token-string">"gpt-5.6-sol"</span>;{"\n"}
          {"    "}<span className="token-keyword">return new</span>{" "}<span className="token-type">Session</span>(projectId, model, <span className="token-boolean">true</span>);{"\n"}
          {"  "}{"}"}{"\n"}
          {"}"}
        </CodeBlock>

        <h3>Go</h3>
        <CodeBlock language="go">
          <span className="token-keyword">package</span> sessions{"\n\n"}
          <span className="token-keyword">import</span>{" "}<span className="token-string">"context"</span>{"\n\n"}
          <span className="token-keyword">func</span>{" "}<span className="token-function">Create</span>(ctx <span className="token-type">context.Context</span>, projectID <span className="token-type">string</span>) (<span className="token-type">Session</span>, <span className="token-type">error</span>) {"{"}{"\n"}
          {"  "}session, err <span className="token-operator">:=</span> store.<span className="token-function">Create</span>(ctx, projectID){"\n"}
          {"  "}<span className="token-keyword">if</span> err <span className="token-operator">!=</span>{" "}<span className="token-boolean">nil</span>{" "}{"{"}{"\n"}
          {"    "}<span className="token-keyword">return</span>{" "}<span className="token-type">Session</span>{"{}"}, err{"\n"}
          {"  "}{"}"}{"\n"}
          {"  "}<span className="token-keyword">return</span> session, <span className="token-boolean">nil</span>{"\n"}
          {"}"}
        </CodeBlock>

        <h3>Rust</h3>
        <CodeBlock language="rust">
          <span className="token-annotation">#[derive(Debug, Clone)]</span>{"\n"}
          <span className="token-keyword">struct</span>{" "}<span className="token-type">Session</span>&lt;<span className="token-lifetime">'a</span>&gt; {"{"}{"\n"}
          {"  "}project_id: <span className="token-operator">&amp;</span><span className="token-lifetime">'a</span>{" "}<span className="token-type">str</span>,{"\n"}
          {"  "}ready: <span className="token-type">bool</span>,{"\n"}
          {"}"}{"\n\n"}
          <span className="token-keyword">fn</span>{" "}<span className="token-function">create_session</span>(project_id: <span className="token-operator">&amp;</span><span className="token-type">str</span>) -&gt; <span className="token-type">Result</span>&lt;<span className="token-type">Session</span>&lt;<span className="token-placeholder">'_</span>&gt;, <span className="token-type">Error</span>&gt; {"{"}{"\n"}
          {"  "}<span className="token-macro">tracing::info!</span>(<span className="token-string">"creating session"</span>);{"\n"}
          {"  "}<span className="token-type">Ok</span>(<span className="token-type">Session</span>{" "}{"{"} project_id, ready: <span className="token-boolean">true</span>{" "}{"}"}){"\n"}
          {"}"}
        </CodeBlock>

        <h3>Python</h3>
        <CodeBlock language="python">
          <span className="token-comment"># Keep only sessions that are ready</span>{"\n"}
          <span className="token-keyword">def</span>{" "}<span className="token-function">ready_sessions</span>(sessions):{"\n"}
          {"    "}<span className="token-keyword">return</span> [{"\n"}
          {"        "}session.id{"\n"}
          {"        "}<span className="token-keyword">for</span> session <span className="token-keyword">in</span> sessions{"\n"}
          {"        "}<span className="token-keyword">if</span> session.status == <span className="token-string">"ready"</span>{"\n"}
          {"    "}] {"\n"}
          {"\n"}
          <span className="token-function">print</span>(<span className="token-function">ready_sessions</span>(sessions))
        </CodeBlock>

        <h3>Bash</h3>
        <CodeBlock language="bash">
          <span className="token-comment"># Open a project and create a session</span>{"\n"}
          <span className="token-function">suncode</span> project open <span className="token-string">"./sample-project"</span>{"\n"}
          <span className="token-keyword">if</span> <span className="token-function">suncode</span> session create --model <span className="token-string">"gpt-5.6-sol"</span>; <span className="token-keyword">then</span>{"\n"}
          {"  "}<span className="token-function">echo</span>{" "}<span className="token-string">"Session ready"</span>{"\n"}
          <span className="token-keyword">fi</span>
        </CodeBlock>

        <h3>HTML</h3>
        <CodeBlock language="html">
          <span className="token-comment">&lt;!-- Session status --&gt;</span>{"\n"}
          <span className="token-punctuation">&lt;</span><span className="token-tag">section</span>{" "}<span className="token-attribute">aria-label</span><span className="token-operator">=</span><span className="token-string">"Session status"</span><span className="token-punctuation">&gt;</span>{"\n"}
          {"  "}<span className="token-punctuation">&lt;</span><span className="token-tag">strong</span><span className="token-punctuation">&gt;</span>Ready<span className="token-punctuation">&lt;/</span><span className="token-tag">strong</span><span className="token-punctuation">&gt;</span>{"\n"}
          <span className="token-punctuation">&lt;/</span><span className="token-tag">section</span><span className="token-punctuation">&gt;</span>
        </CodeBlock>

        <h3>CSS</h3>
        <CodeBlock language="css">
          <span className="token-selector">.session-status</span>{" "}{"{"}{"\n"}
          {"  "}<span className="token-property">display</span><span className="token-punctuation">:</span> grid<span className="token-punctuation">;</span>{"\n"}
          {"  "}<span className="token-property">gap</span><span className="token-punctuation">:</span>{" "}<span className="token-number">8px</span><span className="token-punctuation">;</span>{"\n"}
          {"  "}<span className="token-property">color</span><span className="token-punctuation">:</span>{" "}<span className="token-function">var</span>(<span className="token-variable">--text</span>)<span className="token-punctuation">;</span>{"\n"}
          {"}"}
        </CodeBlock>

        <h3>JSON</h3>
        <CodeBlock language="json">
          {"{"}{"\n"}
          {"  "}<span className="token-property">"project"</span>: <span className="token-string">"suncode"</span>,{"\n"}
          {"  "}<span className="token-property">"sessionCount"</span>: <span className="token-number">3</span>,{"\n"}
          {"  "}<span className="token-property">"ready"</span>: <span className="token-boolean">true</span>{"\n"}
          {"}"}
        </CodeBlock>

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
