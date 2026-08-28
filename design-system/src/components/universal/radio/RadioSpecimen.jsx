import { Specimen } from "../Specimen.jsx";

export function RadioSpecimen() {
  return <Specimen label="Radio"><div className="stack"><label className="radio"><input type="radio" name="scope" defaultChecked /> Current project</label><label className="radio"><input type="radio" name="scope" /> Current session</label></div></Specimen>;
}
