import { Icon } from "../../../shared/Icon.jsx";

export function CardSpecimens() {
  return <div className="specimen-grid specimen-grid-3">
    <div className="card project-card"><span className="project-mark">S</span><div><strong>suncode</strong><small>~/Projects/suncode</small></div><span className="badge success">Ready</span></div>
    <div className="card"><div className="activity"><Icon name="foundation" /><div><h3>Reading project files</h3><p>Scanning <span className="mono">agent/</span> for provider usage.</p></div><small>12s</small></div></div>
    <div className="card warning"><div className="approval"><span className="approval-mark">!</span><div><h3>Approval required</h3><p>Run <span className="mono">cargo test</span> in this project.</p></div><div className="approval-actions"><button className="btn btn-sm">Deny</button><button className="btn btn-primary btn-sm">Approve</button></div></div></div>
  </div>;
}
