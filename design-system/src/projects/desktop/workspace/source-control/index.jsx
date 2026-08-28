import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { SourceControlPanel } from "../WorkspacePrimitives.jsx";

export function WorkspaceSourceControlPage() {
  return <><PageHeader title="Source control" description="Read-only repository status and patch review for the open project." status="Phase 1" tone="implemented" /><Section id="source-control-panel" title="Changed files and diff"><SourceControlPanel standalone /></Section></>;
}
