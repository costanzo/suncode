import { PageHeader, Section } from "../../../../shared/PagePrimitives.jsx";
import { NavigationSpecimens } from "../../navigation/index.js";

export function NavigationPage() {
  return (
    <>
      <PageHeader title="Navigation" description="Universal navigation patterns expose active context without visual noise." path="components/universal/navigation/" status="Universal" tone="implemented" />
      <Section id="navigation-controls" title="Navigation and filters" description="Active context is visible without turning every option into a filled pill."><NavigationSpecimens /></Section>
    </>
  );
}
