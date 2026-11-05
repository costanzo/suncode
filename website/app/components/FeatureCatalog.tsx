"use client";

import { useDeferredValue, useState } from "react";
import { ArrowUpRight, Check, Search, X } from "lucide-react";
import {
  featureGroups,
  features,
  sourceUrl,
  statusLabels,
  type FeatureStatus,
} from "../../lib/features";

export function FeatureCatalog() {
  const [query, setQuery] = useState("");
  const [status, setStatus] = useState("all");
  const search = useDeferredValue(query.trim().toLowerCase());
  const visible = features.filter(
    (feature) =>
      (status === "all" || feature.status === status) &&
      `${feature.title} ${feature.description} ${feature.details.join(" ")} ${feature.note} ${feature.id}`
        .toLowerCase()
        .includes(search),
  );
  const groups = featureGroups.filter((group) =>
    visible.some((feature) => feature.group === group.id),
  );
  function reset() {
    setQuery("");
    setStatus("all");
  }

  return (
    <>
      <div className="catalog-toolbar">
        <div className="catalog-search">
          <Search size={19} aria-hidden="true" />
          <label className="sr-only" htmlFor="feature-search">
            Search features
          </label>
          <input
            id="feature-search"
            type="search"
            placeholder="Find a feature, tool, or workflow…"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            autoComplete="off"
          />
          {query && (
            <button
              className="search-clear"
              type="button"
              aria-label="Clear search"
              onClick={() => setQuery("")}
            >
              <X size={16} />
            </button>
          )}
        </div>
        <div className="catalog-filter">
          <label htmlFor="feature-status">Availability</label>
          <select
            id="feature-status"
            value={status}
            onChange={(event) => setStatus(event.target.value)}
          >
            <option value="all">All capabilities</option>
            {Object.entries(statusLabels).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </div>
      </div>
      <div className="catalog-layout">
        <aside className="catalog-index">
          <nav aria-label="Feature categories">
            <p>In this guide</p>
            {groups.map((group) => (
              <a href={`#${group.id}`} key={group.id}>
                {group.shortTitle}
                <span>
                  {
                    visible.filter((feature) => feature.group === group.id)
                      .length
                  }
                </span>
              </a>
            ))}
          </nav>
          <p className="catalog-index-note">
            Implementation describes source capabilities. Preview and release
            checks are shown where they affect availability.
          </p>
          <a
            className="text-link"
            href={`${sourceUrl}/.agents/features`}
            target="_blank"
            rel="noreferrer"
          >
            Technical feature records <ArrowUpRight size={14} />
          </a>
        </aside>
        <div className="catalog-content">
          <div className="catalog-result" role="status" aria-live="polite">
            {visible.length}{" "}
            {visible.length === 1 ? "capability" : "capabilities"}
            {query || status !== "all"
              ? " matching your filters"
              : " across the product and its tooling"}
          </div>
          {visible.length === 0 ? (
            <div className="catalog-empty">
              <Search size={30} />
              <h2>No features found.</h2>
              <p>
                Try “CLI”, “documents”, or a broader search. You can also clear
                the availability filter.
              </p>
              <button
                className="button button-primary"
                type="button"
                onClick={reset}
              >
                Clear all filters
              </button>
            </div>
          ) : (
            groups.map((group) => (
              <section className="feature-group" id={group.id} key={group.id}>
                <header>
                  <h2>{group.title}</h2>
                  <p>{group.description}</p>
                </header>
                <div>
                  {visible
                    .filter((feature) => feature.group === group.id)
                    .map((feature) => (
                      <article
                        className="feature-entry"
                        id={feature.id}
                        key={feature.id}
                      >
                        <div className="feature-entry-heading">
                          <h3>{feature.title}</h3>
                          <span
                            className={`availability availability-${feature.status}`}
                          >
                            {statusLabels[feature.status as FeatureStatus]}
                          </span>
                        </div>
                        <p className="feature-description">
                          {feature.description}
                        </p>
                        <ul>
                          {feature.details.map((detail) => (
                            <li key={detail}>
                              <Check size={15} aria-hidden="true" />
                              <span>{detail}</span>
                            </li>
                          ))}
                        </ul>
                        <details className="feature-limits">
                          <summary>Scope & availability</summary>
                          <div>
                            <p>{feature.note}</p>
                            <a
                              href={`${sourceUrl}/.agents/features/${feature.id}/README.md`}
                              target="_blank"
                              rel="noreferrer"
                            >
                              Read technical notes{" "}
                              <ArrowUpRight size={13} aria-hidden="true" />
                            </a>
                          </div>
                        </details>
                      </article>
                    ))}
                </div>
              </section>
            ))
          )}
          <section className="catalog-closing">
            <h2>A local agent, with explicit connections.</h2>
            <p>
              Your project work and session state live locally. Model requests
              send selected context to configured providers. Optional tools and
              paired remote control have their own network and authority
              boundaries.
            </p>
            <p>
              Web, TUI, IDE-plugin clients, hosted execution, and
              Python/TypeScript bindings remain future work. Package and
              platform support should be checked against the source guides
              before installation.
            </p>
          </section>
        </div>
      </div>
    </>
  );
}
