import { Icon } from "../../../shared/Icon.jsx";
import { Specimen } from "../Specimen.jsx";

export function BadgeSpecimens() {
  return <Specimen label="Badges and alerts"><div className="row"><span className="badge accent">Running</span><span className="badge success">Connected</span><span className="badge warning">Waiting approval</span><span className="badge danger">Failed</span></div><div className="stack feedback-stack"><Alert tone="success" title="Checkpoint created">This turn can be undone from review.</Alert><Alert tone="warning" title="External side effect">Process output cannot be reverted.</Alert><Alert tone="danger" title="Provider unavailable">Check credentials or choose another model.</Alert></div></Specimen>;
}

function Alert({ tone, title, children }) {
  return <div className={`alert ${tone}`}><Icon name={tone === "danger" ? "close" : "check"} /><div><strong>{title}</strong><span>{children}</span></div></div>;
}
