import { ModuleLink, PageHeader, Section } from "../../shared/PagePrimitives.jsx";

export function WebPlatformPage() {
  return <><PageHeader title="Web platform" description="A reserved adaptation layer for a future browser client." status="Module index" tone="deferred" showOwnership={false} /><Section id="module-index" title="Web modules"><div className="module-card-grid"><ModuleLink to="/platforms/web/boundary" icon="platform" title="Adaptation boundary" description="What is reserved without implying implementation." /><ModuleLink to="/platforms/web/ownership" icon="platform" title="Ownership contract" description="Where future web tokens, components, and pages belong." /></div></Section></>;
}
