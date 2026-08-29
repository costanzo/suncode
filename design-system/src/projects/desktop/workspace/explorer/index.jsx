import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { ExplorerPanel } from "../WorkspacePrimitives.jsx";

export function WorkspaceExplorerPage() {
  return <><PageHeader title="Explorer" description="Project files and registered read-only dependency roots within the current project boundary." /><Section id="explorer-panel" title="Project explorer"><div className="workspace-state-grid workspace-explorer-state-grid"><div><h3>Without dependencies</h3><ExplorerPanel standalone hasDependency={false} /></div><div><h3>With dependencies</h3><ExplorerPanel standalone hasDependency /></div><div><h3>Long paths and deep nesting</h3><ExplorerPanel standalone hasDependency constrained /></div></div></Section></>;
}
