import { FileTree, PageHeader, Section } from "../shared/PagePrimitives.jsx";
import { Icon } from "../shared/Icon.jsx";

export function DeferredBoundaryPage({ platform, title, icon }) {
  return <><PageHeader title={`${title} adaptation boundary`} description={`The ${title} client is deferred; this page records the reserved boundary without implying implementation.`} path={`platforms/${platform}/boundary/`} status="Deferred" tone="deferred" /><Section id="reserved-boundary" title="Reserved adaptation boundary"><div className="deferred-panel"><span className="deferred-icon"><Icon name={icon} size={28} /></span><div><h3>No component surface</h3><p>Future work belongs here only after the client direction is approved. This catalog does not invent unapproved controls.</p></div></div></Section></>;
}

export function DeferredOwnershipPage({ platform, title }) {
  return <><PageHeader title={`${title} ownership contract`} description={`The reserved source boundaries for a future ${title} adaptation.`} path={`platforms/${platform}/ownership/`} status="Deferred" tone="deferred" /><Section id="ownership-tree" title="Ownership contract"><FileTree>{`platforms/${platform}/\n├── index.jsx\n├── tokens/\n├── components/\n${platform === "mobile" ? "├── pages/\n├── styles/\n" : ""}└── overrides.css`}</FileTree></Section></>;
}
