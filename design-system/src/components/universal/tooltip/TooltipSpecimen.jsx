import { Icon } from "../../../shared/Icon.jsx";
import { Specimen } from "../Specimen.jsx";

export function TooltipSpecimen() {
  return (
    <Specimen label="Tooltip">
      <div className="tooltip-demo">
        <button className="btn btn-icon" aria-label="Copy path">
          <Icon name="assets" />
        </button>
        <span role="tooltip">Copy project path</span>
      </div>
      <p className="sample-note">Hover or focus the icon action.</p>
    </Specimen>
  );
}
