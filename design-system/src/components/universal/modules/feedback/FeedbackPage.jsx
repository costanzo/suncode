import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { BadgeSpecimens } from "../../badge/index.js";
import { FeedbackSpecimen } from "../../feedback/index.js";

export function FeedbackPage() {
  return (
    <>
      <PageHeader
        title="Feedback"
        description="Universal feedback communicates health, authority, failure, and progress with restrained semantic color."
        path="components/universal/feedback/"
        status="Universal"
        tone="implemented"
      />
      <Section
        id="feedback-states"
        title="Status, alerts, and progress"
        description="Semantic color reports health, authority, failure, or active work only."
      >
        <div className="specimen-grid specimen-grid-2">
          <BadgeSpecimens />
          <FeedbackSpecimen />
        </div>
      </Section>
    </>
  );
}
