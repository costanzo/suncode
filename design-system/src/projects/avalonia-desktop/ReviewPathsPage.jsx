import { ModuleLink, PageHeader, Section } from "../../shared/PagePrimitives.jsx";

export function ReviewPathsPage() {
  return <><PageHeader title="Review paths" description="The approved references used when mapping design decisions into Avalonia." path="projects/avalonia-desktop/review-paths/" status="Production mapping" tone="implemented" /><Section id="references" title="Review paths"><div className="layer-list"><ModuleLink to="/components/universal" icon="components" title="Universal inventory" description="Confirm primitive semantics and states." status="Review" tone="review" /><ModuleLink to="/platforms/desktop" icon="platform" title="Desktop adaptation" description="Confirm shell anatomy and desktop-only behavior." status="Phase 1" tone="implemented" /><ModuleLink to="/core/tokens" icon="foundation" title="Token mapping" description="Keep Avalonia resource names semantically aligned." status="Source" tone="review" /></div></Section></>;
}
