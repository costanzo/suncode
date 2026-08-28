import { PageHeader, Section } from "../../../shared/PagePrimitives.jsx";

export function FontsPage() {
  return <><PageHeader title="Fonts" description="Typeface guidance for readable product UI and precise technical content." path="core/assets/fonts/" status="Asset guidance" tone="review" /><Section id="font-stacks" title="Font stacks" description="Runtime packaging must license and include a font before depending on it."><div className="font-contract"><p><strong>UI</strong><span>Noto Sans · Noto Sans CJK SC · PingFang SC · Helvetica Neue</span></p><p><strong>Code / data</strong><span>JetBrains Mono · SF Mono · Menlo · Consolas</span></p></div></Section></>;
}
