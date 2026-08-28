import { Icon } from "../../../shared/Icon.jsx";
import { Specimen } from "../Specimen.jsx";

export function FeedbackSpecimen() {
  return <Specimen label="Loading and empty"><div className="split"><span className="type-label">Analyzing repository</span><span className="mono type-caption">62%</span></div><div className="progress"><span /></div><div className="skeleton" style={{ width: "84%" }} /><div className="skeleton short" /><div className="empty"><div><Icon name="assets" /><strong>No changes yet</strong><span>Files touched by this turn will appear here.</span></div></div></Specimen>;
}
