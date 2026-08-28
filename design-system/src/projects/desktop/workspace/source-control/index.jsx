import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { SourceControlPanel } from "../WorkspacePrimitives.jsx";

export function WorkspaceSourceControlPage() {
  return <><PageHeader title="Source control" description="Read-only repository status and patch review for the open project." /><Section id="source-control-panel" title="Changed files and diff"><div className="workspace-state-grid"><div><h3>Working tree clean</h3><SourceControlPanel standalone clean /></div><div><h3>Changed files</h3><SourceControlPanel standalone /></div></div></Section></>;
}
