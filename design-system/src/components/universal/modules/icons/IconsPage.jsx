import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { FileIconSpecimens } from "../../file-icon/index.js";

export function IconsModulePage() {
  return (
    <>
      <PageHeader
        title="File icons"
        description="Seti-derived glyphs identify file type in dense resource lists while labels remain the primary identifier."
        status="Universal"
        tone="implemented"
      />
      <Section
        id="file-icons"
        title="Explorer file types"
        description="Filename rules take precedence over compound extensions, which take precedence over a detected language or the generic file glyph."
      >
        <FileIconSpecimens />
      </Section>
    </>
  );
}
