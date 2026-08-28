import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { ProviderTracePanel } from "../WorkspacePrimitives.jsx";

export function WorkspaceProviderTracePage() {
  return <><PageHeader title="Provider trace" description="Model exchanges, canonical content, tool activity, usage, timing, and redacted provider identifiers." /><Section id="provider-trace-panel" title="Model exchange detail"><div className="workspace-state-grid"><div><h3>No turns</h3><ProviderTracePanel standalone state="no-turns" /></div><div><h3>Turn collapsed</h3><ProviderTracePanel standalone state="turn-collapsed" /></div><div><h3>Turn expanded</h3><ProviderTracePanel standalone state="turn-expanded" /></div><div><h3>Exchange expanded</h3><ProviderTracePanel standalone state="expanded" /></div></div></Section></>;
}
