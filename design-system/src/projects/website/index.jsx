import { useState } from "react";
import { BrandMark } from "../../shared/BrandMark.jsx";
import { Icon } from "../../shared/Icon.jsx";
import { PageHeader, Section } from "../../shared/PagePrimitives.jsx";
import "./website.css";

const samples = [
  {
    title: "Project-aware conversations",
    status: "Implemented",
    description: "Work through persistent sessions with visible tool activity.",
    note: "Queued follow-ups stay in memory until admitted to history.",
  },
  {
    title: "Browser Use and local preview",
    status: "Preview",
    description: "Optional CEF-enabled desktop builds can navigate and interact with pages.",
    note: "CEF is excluded from default builds and baseline installers.",
  },
  {
    title: "Independent component updates",
    status: "Release verification",
    description: "Check signed metadata and plan compatible Desktop and SDK updates.",
    note: "Real package archives and trusted installation receipts remain pending.",
  },
  {
    title: "An executable design system",
    status: "Developer tooling",
    description: "Inspect shared components and interaction states.",
    note: "Review tooling is separate from released agent clients.",
  },
];

export function WebsiteProjectPage() {
  const [narrow, setNarrow] = useState(false);
  const [theme, setTheme] = useState("light");
  const [query, setQuery] = useState("");
  const [availability, setAvailability] = useState("All capabilities");
  const [step, setStep] = useState(0);
  const [menu, setMenu] = useState(false);
  const visible = samples.filter(
    (sample) =>
      (availability === "All capabilities" || sample.status === availability) &&
      `${sample.title} ${sample.description}`.toLowerCase().includes(query.toLowerCase()),
  );
  return (
    <>
      <PageHeader
        title="Official website"
        description="Overview and complete feature guide for suncode.ai. This is a product information site, separate from the Web remote-control specimen."
        status="Review reference"
        tone="implemented"
      />
      <Section
        title="Composition and navigation"
        description="Light paper/slate and dark graphite/silver themes share the same composition. The header theme control switches both specimens. Production follows the system until a manual choice is saved."
      >
        <label className="website-review-toggle">
          <input
            type="checkbox"
            checked={narrow}
            onChange={(event) => setNarrow(event.target.checked)}
          />{" "}
          Constrained layout
        </label>
        <div
          className={`website-specimen ${narrow ? "website-specimen-narrow" : ""}`}
          data-website-theme={theme}
        >
          <header className="website-specimen-nav">
            <strong>
              <BrandMark size={24} /> SunCode
            </strong>
            {!narrow && <span>Overview　 Features　 How it works</span>}
            <button
              className="website-specimen-theme"
              type="button"
              aria-label={
                theme === "light" ? "Switch specimen to dark mode" : "Switch specimen to light mode"
              }
              onClick={() => setTheme(theme === "light" ? "dark" : "light")}
            >
              <Icon name={theme === "light" ? "moon" : "sun"} />
            </button>
            {narrow && (
              <button type="button" aria-expanded={menu} onClick={() => setMenu(!menu)}>
                {menu ? "Close menu" : "Open menu"}
              </button>
            )}
          </header>
          {narrow && menu && (
            <nav className="website-specimen-menu">
              <a href="#/projects/website">Overview</a>
              <a href="#/projects/website">Features</a>
            </nav>
          )}
          <div className="website-specimen-home">
            <h2>
              Let the agent work.
              <br />
              <span>Keep the work in view.</span>
            </h2>
            <p>A coding agent for understanding, changing, and reviewing your projects.</p>
            <a className="website-specimen-cta" href="#/projects/website">
              Explore every feature
            </a>
            <div className="website-specimen-tabs">
              {["Inspect the project", "Review the scope", "Inspect the changes"].map(
                (label, index) => (
                  <button
                    key={label}
                    type="button"
                    aria-pressed={step === index}
                    onClick={() => setStep(index)}
                  >
                    {label}
                  </button>
                ),
              )}
            </div>
            <div className="website-specimen-demo">
              <span>Illustrative session</span>
              <h3>
                {
                  ["Understand before changing", "Approval needed", "Checkpointed file changes"][
                    step
                  ]
                }
              </h3>
              <p>
                {
                  [
                    "Read project context and tool activity.",
                    "Inspect the operation and its scope before deciding.",
                    "Review mutations and restore with conflict checks.",
                  ][step]
                }
              </p>
            </div>
          </div>
        </div>
      </Section>
      <Section
        title="Feature guide and availability"
        description="Search, result counts, empty recovery, status labels, and keyboard-operable scope disclosure. Production includes all 31 source feature records in seven groups."
      >
        <div
          className={`website-specimen ${narrow ? "website-specimen-narrow" : ""}`}
          data-website-theme={theme}
        >
          <div className="website-specimen-guide">
            <label htmlFor="website-specimen-search">Search features</label>
            <input
              id="website-specimen-search"
              type="search"
              placeholder="Try browser or a missing term"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
            />
            <label htmlFor="website-specimen-availability">Availability</label>
            <select
              id="website-specimen-availability"
              value={availability}
              onChange={(event) => setAvailability(event.target.value)}
            >
              {["All capabilities", ...samples.map((sample) => sample.status)].map((status) => (
                <option key={status}>{status}</option>
              ))}
            </select>
            <p className="website-specimen-result" role="status">
              {visible.length} matching capabilities
            </p>
            {visible.length ? (
              visible.map((sample) => (
                <article key={sample.title}>
                  <div>
                    <h3>{sample.title}</h3>
                    <span data-availability={sample.status}>{sample.status}</span>
                  </div>
                  <p>{sample.description}</p>
                  <details>
                    <summary>Scope & availability</summary>
                    <p>{sample.note}</p>
                  </details>
                </article>
              ))
            ) : (
              <div className="website-specimen-empty">
                <h3>No features found.</h3>
                <p>Try a broader search or clear the filters.</p>
                <button
                  type="button"
                  onClick={() => {
                    setQuery("");
                    setAvailability("All capabilities");
                  }}
                >
                  Clear all filters
                </button>
              </div>
            )}
          </div>
        </div>
      </Section>
    </>
  );
}
