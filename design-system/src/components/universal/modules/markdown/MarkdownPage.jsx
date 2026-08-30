import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { MarkdownSpecimen } from "../../markdown/index.js";

export function MarkdownPage() {
  return (
    <>
      <PageHeader
        title="Markdown"
        description="Universal assistant output needs a complete, readable structure for technical communication."
        path="components/universal/markdown/"
        status="Universal"
        tone="implemented"
      />
      <Section
        id="markdown-surface"
        title="Markdown reading surface"
        description="Assistant content keeps a readable measure and complete structural hierarchy."
      >
        <MarkdownSpecimen />
      </Section>
    </>
  );
}
