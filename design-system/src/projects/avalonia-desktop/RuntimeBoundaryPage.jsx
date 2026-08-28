import { PageHeader, Section } from "../../shared/PagePrimitives.jsx";

export function RuntimeBoundaryPage() {
  return <><PageHeader title="Runtime boundary" description="The production ownership split across presentation, agent behavior, and review tooling." path="projects/avalonia-desktop/runtime-boundary/" status="Production mapping" tone="implemented" /><Section id="boundary" title="Runtime boundary"><div className="boundary-note"><strong>Production remains native.</strong><p>Avalonia owns presentation and navigation. The embedded Rust SDK owns providers, sessions, policy, persistence, credentials, operations, recovery, and undo. This React app ships no product client behavior.</p><code>apps/desktop-avalonia/App.axaml</code></div></Section></>;
}
