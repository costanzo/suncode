import { Specimen } from "../Specimen.jsx";
import { Radio } from "./Radio.jsx";

export function RadioSpecimen() {
  return (
    <Specimen label="Radio">
      <div className="stack">
        <label className="radio">
          <Radio name="scope" value="project" defaultChecked /> Current project
        </label>
        <label className="radio">
          <Radio name="scope" value="session" /> Current session
        </label>
      </div>
    </Specimen>
  );
}
