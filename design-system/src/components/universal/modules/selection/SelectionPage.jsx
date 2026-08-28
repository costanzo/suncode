import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { CheckboxSpecimen } from "../../checkbox/index.js";
import { RadioSpecimen } from "../../radio/index.js";
import { ToggleSpecimen } from "../../toggle/index.js";
import { DropdownSpecimens } from "../../dropdown/index.js";

export function SelectionPage() {
  return (
    <>
      <PageHeader title="Selection" description="Universal selection controls preserve familiar input behavior and visible state." path="components/universal/selection/" status="Universal" tone="implemented" />
      <Section id="selection-controls" title="Selection controls" description="Native inputs retain their familiar behavior and visible focus."><div className="specimen-grid specimen-grid-3"><CheckboxSpecimen /><RadioSpecimen /><ToggleSpecimen /></div></Section>
      <Section id="dropdowns" title="Dropdowns" description="Flat choices and provider-grouped models use the same compact menu treatment."><DropdownSpecimens /></Section>
    </>
  );
}
