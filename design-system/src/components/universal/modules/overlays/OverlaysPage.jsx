import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { AvatarSpecimen } from "../../avatar/index.js";
import { ModalSpecimen } from "../../modal/index.js";
import { TooltipSpecimen } from "../../tooltip/index.js";

export function OverlaysPage() {
  return (
    <>
      <PageHeader
        title="Overlays"
        description="Contextual layers, confirmation dialogs, and identity markers appear only when they clarify focus or ownership."
        path="components/universal/overlays/"
        status="Universal"
        tone="implemented"
      />
      <Section
        id="overlay-components"
        title="Avatar, dialog, and tooltip"
        description="Overlays appear only when focus or compact explanation genuinely requires them."
      >
        <div className="specimen-grid specimen-grid-3">
          <AvatarSpecimen />
          <ModalSpecimen />
          <TooltipSpecimen />
        </div>
      </Section>
    </>
  );
}
