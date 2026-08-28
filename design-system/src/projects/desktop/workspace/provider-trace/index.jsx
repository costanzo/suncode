import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { ProviderTracePanel } from "../WorkspacePrimitives.jsx";

export function WorkspaceProviderTracePage() {
  return <><PageHeader title="Provider trace" description="Model exchanges, canonical content, tool activity, usage, timing, and redacted provider identifiers." status="Phase 1" tone="implemented" /><Section id="provider-trace-panel" title="Model exchange detail"><ProviderTracePanel standalone /></Section></>;
}
