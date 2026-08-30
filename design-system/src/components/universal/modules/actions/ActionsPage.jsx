import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { ButtonSpecimens } from "../../button/index.js";

export function ActionsPage() {
  return (
    <>
      <PageHeader
        title="Actions"
        description="Universal actions establish clear hierarchy for progress, cancellation, and protected decisions."
        path="components/universal/actions/"
        status="Universal"
        tone="implemented"
      />
      <Section
        id="buttons"
        title="Buttons and actions"
        description="One primary action advances the turn; quiet and destructive actions stay subordinate."
      >
        <ButtonSpecimens />
      </Section>
    </>
  );
}
