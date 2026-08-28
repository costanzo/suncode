import { ModuleLink, PageHeader, Section } from "../../shared/PagePrimitives.jsx";

export function MobilePlatformPage() {
  return <><PageHeader title="Mobile platform" description="A reserved adaptation layer for a future mobile client." status="Module index" tone="deferred" /><Section id="module-index" title="Mobile modules"><div className="module-card-grid"><ModuleLink to="/platforms/mobile/boundary" icon="mobile" title="Adaptation boundary" description="What is reserved without implying implementation." /><ModuleLink to="/platforms/mobile/ownership" icon="mobile" title="Ownership contract" description="Where future mobile tokens, components, and pages belong." /></div></Section></>;
}
