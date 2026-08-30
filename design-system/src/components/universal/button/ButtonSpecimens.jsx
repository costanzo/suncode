import { Icon } from "../../../shared/Icon.jsx";
import { Specimen } from "../Specimen.jsx";

export function ButtonSpecimens() {
  return (
    <div className="specimen-grid specimen-grid-2">
      <Specimen label="Variants">
        <div className="row">
          <button className="btn btn-primary">
            <Icon name="arrow" />
            Send turn
          </button>
          <button className="btn">Cancel</button>
          <button className="btn btn-danger">Delete project</button>
          <button className="btn btn-quiet">Learn more</button>
        </div>
      </Specimen>
      <Specimen label="Size and availability">
        <div className="row">
          <button className="btn btn-primary btn-lg">Open project</button>
          <button className="btn btn-sm">Compact</button>
          <button className="btn btn-icon" aria-label="Settings">
            <Icon name="components" />
          </button>
          <button className="btn btn-icon" disabled aria-label="Disabled">
            <Icon name="close" />
          </button>
        </div>
      </Specimen>
      <Specimen label="State reference">
        <div className="state-strip">
          <div className="state-box">Rest</div>
          <div className="state-box hover">Hover</div>
          <div className="state-box active">Pressed</div>
          <div className="state-box disabled">Disabled</div>
        </div>
      </Specimen>
      <Specimen label="Authority decision">
        <div className="split">
          <span className="type-label">Approve this operation?</span>
          <div className="row">
            <button className="btn btn-sm">Deny</button>
            <button className="btn btn-primary btn-sm">Approve once</button>
          </div>
        </div>
      </Specimen>
    </div>
  );
}
