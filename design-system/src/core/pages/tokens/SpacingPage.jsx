import { PageHeader, Section } from "../../../shared/PagePrimitives.jsx";

const measurements = [
  ["space-1", "4px"],
  ["space-2", "8px"],
  ["space-4", "16px"],
  ["space-6", "24px"],
  ["radius-sm", "6px"],
  ["radius-md", "10px"],
  ["radius-lg", "14px"],
  ["control-md", "36px"],
];

export function SpacingPage() {
  return (
    <>
      <PageHeader
        title="Spacing & shape"
        description="Shared measurements create consistent rhythm, density, and control geometry."
        path="core/tokens/spacing/"
        status="Token source"
        tone="review"
      />
      <Section id="measurements" title="Spacing, shape, and controls">
        <div className="measurement-grid">
          {measurements.map(([name, value]) => (
            <div key={name}>
              <code>{name}</code>
              <strong>{value}</strong>
            </div>
          ))}
        </div>
      </Section>
    </>
  );
}
