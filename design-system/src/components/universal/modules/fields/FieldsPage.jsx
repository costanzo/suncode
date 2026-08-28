import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { InputSpecimens } from "../../input/index.js";

export function FieldsPage() {
  return (
    <>
      <PageHeader title="Fields" description="Universal entry controls keep labels, values, and recovery guidance stable across platforms." path="components/universal/fields/" status="Universal" tone="implemented" />
      <Section id="inputs" title="Input fields" description="Labels stay visible; validation explains how to recover without moving the page."><InputSpecimens /></Section>
    </>
  );
}
