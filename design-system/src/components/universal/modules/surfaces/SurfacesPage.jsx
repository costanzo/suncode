import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { CardSpecimens } from "../../card/index.js";

export function SurfacesPage() {
  return (
    <>
      <PageHeader title="Surfaces" description="Universal surfaces group repeated content and important authority context." path="components/universal/surfaces/" status="Universal" tone="implemented" />
      <Section id="cards" title="Cards and authority surfaces" description="Containers frame repeated content or important tools; they do not become the page scaffold."><CardSpecimens /></Section>
    </>
  );
}
