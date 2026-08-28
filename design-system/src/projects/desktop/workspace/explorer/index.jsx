import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { ExplorerPanel } from "../WorkspacePrimitives.jsx";

export function WorkspaceExplorerPage() {
  return <><PageHeader title="Explorer" description="Project files and registered read-only dependency roots within the current project boundary." status="Phase 1" tone="implemented" /><Section id="explorer-panel" title="Project explorer"><ExplorerPanel standalone /></Section></>;
}
