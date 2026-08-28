import { PageHeader, Section } from "../../shared/PagePrimitives.jsx";
import { AvatarSpecimen } from "./avatar/index.js";
import { BadgeSpecimens } from "./badge/index.js";
import { ButtonSpecimens } from "./button/index.js";
import { CardSpecimens } from "./card/index.js";
import { CheckboxSpecimen } from "./checkbox/index.js";
import { DataSpecimens } from "./data-table/index.js";
import { FeedbackSpecimen } from "./feedback/index.js";
import { InputSpecimens } from "./input/index.js";
import { MarkdownSpecimen } from "./markdown/index.js";
import { ModalSpecimen } from "./modal/index.js";
import { NavigationSpecimens } from "./navigation/index.js";
import { RadioSpecimen } from "./radio/index.js";
import { ToggleSpecimen } from "./toggle/index.js";
import { TooltipSpecimen } from "./tooltip/index.js";

export function UniversalComponentsPage() {
  return (
    <>
      <PageHeader title="Universal components" description="Cross-platform primitives with the same semantic inventory in light and dark themes. Interact with controls here before mapping them into a client." path="components/universal/" status="Complete inventory" tone="implemented" />
      <Section id="actions" title="Buttons and actions" description="One primary action advances the turn; quiet and destructive actions stay subordinate."><ButtonSpecimens /></Section>
      <Section id="fields" title="Fields" description="Labels stay visible; validation explains how to recover without moving the page."><InputSpecimens /></Section>
      <Section id="selection" title="Selection controls" description="Native inputs retain their familiar behavior and visible focus."><div className="specimen-grid specimen-grid-3"><CheckboxSpecimen /><RadioSpecimen /><ToggleSpecimen /></div></Section>
      <Section id="surfaces" title="Cards and authority surfaces" description="Containers frame repeated content or important tools; they do not become the page scaffold."><CardSpecimens /></Section>
      <Section id="overlays" title="Avatar, modal, and tooltip" description="Overlays appear only when focus or compact explanation genuinely requires them."><div className="specimen-grid specimen-grid-3"><AvatarSpecimen /><ModalSpecimen /><TooltipSpecimen /></div></Section>
      <Section id="navigation" title="Navigation and filters" description="Active context is visible without turning every option into a filled pill."><NavigationSpecimens /></Section>
      <Section id="feedback" title="Status, alerts, and progress" description="Semantic color reports health, authority, failure, or active work only."><div className="specimen-grid specimen-grid-2"><BadgeSpecimens /><FeedbackSpecimen /></div></Section>
      <Section id="data" title="Code and data" description="Monospace appears where character precision changes understanding."><DataSpecimens /></Section>
      <Section id="markdown" title="Markdown reading surface" description="Assistant content keeps a readable measure and complete structural hierarchy."><MarkdownSpecimen /></Section>
    </>
  );
}
