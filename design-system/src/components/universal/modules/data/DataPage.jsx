import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { DataSpecimens } from "../../data-table/index.js";

export function DataPage() {
  return (
    <>
      <PageHeader title="Data" description="Universal technical content favors precision, comparison, and readable density." path="components/universal/data/" status="Universal" tone="implemented" />
      <Section id="data-display" title="Code and data" description="Monospace appears where character precision changes understanding."><DataSpecimens /></Section>
    </>
  );
}
