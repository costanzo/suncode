import Link from "next/link";
import {
  ArrowRight,
  ArrowUpRight,
  Check,
  Code2,
  GitBranch,
  Layers3,
  LockKeyhole,
  Terminal,
} from "lucide-react";
import { WorkbenchDemo } from "./components/WorkbenchDemo";
import { BrandMark } from "./components/BrandMark";
import { githubUrl, sourceUrl } from "../lib/features";

export default function Home() {
  return (
    <main id="main-content">
      <section className="home-hero shell">
        <h1>
          Let the agent work.
          <br />
          <span>Keep the work in view.</span>
        </h1>
        <div className="hero-bottom">
          <p>
            A coding agent for understanding, changing, and reviewing your
            projects. Native desktop. Visible actions. Decisions that stay
            yours.
          </p>
          <div className="hero-actions">
            <Link className="button button-primary" href="/features">
              Explore every feature <ArrowRight size={17} />
            </Link>
            <a
              className="text-link"
              href={githubUrl}
              target="_blank"
              rel="noreferrer"
            >
              View on GitHub <ArrowUpRight size={16} />
            </a>
          </div>
        </div>
        <WorkbenchDemo />
      </section>
      <section className="principles shell" aria-label="Product principles">
        <p>
          <Code2 size={19} /> One embedded Rust agent
        </p>
        <p>
          <LockKeyhole size={19} /> Scoped approvals
        </p>
        <p>
          <GitBranch size={19} /> Checkpointed file undo
        </p>
      </section>
      <section className="home-intro shell" id="why">
        <h2>
          More than an answer.
          <br />
          <span>A way to move the project forward.</span>
        </h2>
        <p>
          From finding the right file to making and checking a change, SunCode
          keeps the conversation close to the tools. Follow the activity,
          inspect the scope, and decide what happens next.
        </p>
      </section>
      <section
        className="capability-stories shell"
        aria-label="Core capabilities"
      >
        <article className="story-row">
          <div className="story-title">
            <Layers3 size={24} />
            <h3>
              Context that
              <br />
              stays connected.
            </h3>
          </div>
          <div>
            <p>
              Work in project sessions with repository guidance, local Skills,
              image attachments, and automatic context compaction. Bring in
              language-server insights and document content when the work needs
              them.
            </p>
            <Link className="text-link" href="/features#integrations">
              Explore tools & integrations <ArrowRight size={15} />
            </Link>
          </div>
          <div className="story-fragment context-fragment">
            <div>
              <FileLabel name="AGENTS.md" />
              <span>Project guidance</span>
            </div>
            <div>
              <FileLabel name="design.docx" />
              <span>Document context</span>
            </div>
            <div>
              <FileLabel name="definition" />
              <span>Language server</span>
            </div>
          </div>
        </article>
        <article className="story-row">
          <div className="story-title">
            <LockKeyhole size={24} />
            <h3>
              Authority you
              <br />
              can inspect.
            </h3>
          </div>
          <div>
            <p>
              Tool activity is visible as work happens. Review sensitive actions
              before they run, inspect touched files, and restore checkpointed
              file edits with conflict checks.
            </p>
            <Link className="text-link" href="/features#rust-core-phase-1">
              See approvals & undo <ArrowRight size={15} />
            </Link>
          </div>
          <div className="story-fragment authority-fragment">
            <LockKeyhole size={18} />
            <strong>An explicit decision.</strong>
            <p>
              Action, scope, and outcome
              <br />
              in the same view.
            </p>
            <span>
              <Check size={14} /> Your approval comes first
            </span>
          </div>
        </article>
        <article className="story-row">
          <div className="story-title">
            <Terminal size={24} />
            <h3>
              Your workflow.
              <br />
              The same core.
            </h3>
          </div>
          <div>
            <p>
              Use the native desktop, run one-shot turns from the CLI, or embed
              the Rust SDK. The{" "}
              <Link href="/features#mobile-client">
                preview mobile companion
              </Link>{" "}
              can follow sessions and answer requests while your desktop and
              relay remain online.
            </p>
            <Link className="text-link" href="/features#automation">
              Explore CLI & SDK <ArrowRight size={15} />
            </Link>
          </div>
          <div className="story-fragment terminal-fragment">
            <div>
              <span />
              <span />
              <span />
            </div>
            <code>
              <span>$</span> suncode run .<br />
              &nbsp; --prompt &quot;Review this change&quot;
            </code>
            <small>One project. One turn. Typed events.</small>
          </div>
        </article>
      </section>
      <section className="workflow-section shell" id="workflow">
        <div className="section-heading">
          <h2>
            From a request
            <br />
            to a reviewed change.
          </h2>
          <p>
            A clear working loop, with room for your judgment at every step.
          </p>
        </div>
        <ol className="workflow-steps">
          {[
            [
              "Open your project",
              "Start a session against the directory you want to work in.",
            ],
            [
              "Give the direction",
              "Describe the outcome. Add context, answer questions, or queue a follow-up.",
            ],
            [
              "Follow and decide",
              "Inspect tool activity and approve sensitive operations with scope in view.",
            ],
            [
              "Review the result",
              "Inspect files and current Git diffs. Restore checkpointed file changes if needed.",
            ],
          ].map(([title, description]) => (
            <li key={title}>
              <h3>{title}</h3>
              <p>{description}</p>
            </li>
          ))}
        </ol>
        <p className="workflow-boundary">
          File undo applies to checkpointed mutations. Shell commands and
          external tool effects have separate boundaries.
        </p>
      </section>
      <section className="open-section shell">
        <div className="open-symbol" aria-hidden="true">
          <BrandMark size={112} />
        </div>
        <div>
          <h2>
            See what’s possible.
            <br />
            <span>See how it’s built.</span>
          </h2>
          <p>
            The complete feature guide covers the desktop, agent, integrations,
            CLI, SDK, and preview capabilities—with their current limits.
          </p>
          <div className="open-actions">
            <Link className="button button-primary" href="/features">
              Browse all features <ArrowRight size={17} />
            </Link>
            <a
              className="text-link"
              href={sourceUrl}
              target="_blank"
              rel="noreferrer"
            >
              Explore the source <ArrowUpRight size={16} />
            </a>
          </div>
        </div>
      </section>
    </main>
  );
}

function FileLabel({ name }: { name: string }) {
  return (
    <code>
      <Code2 size={13} />
      {name}
    </code>
  );
}
