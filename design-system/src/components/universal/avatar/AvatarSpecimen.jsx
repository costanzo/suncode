import { Specimen } from "../Specimen.jsx";

export function AvatarSpecimen() {
  return (
    <Specimen label="Avatar group">
      <div className="avatar-group" aria-label="Three collaborators">
        <span className="avatar avatar-silver">S</span>
        <span className="avatar avatar-steel">R</span>
        <span className="avatar avatar-neutral">A</span>
      </div>
      <p className="sample-note">Initials are a fallback when no approved image exists.</p>
    </Specimen>
  );
}
