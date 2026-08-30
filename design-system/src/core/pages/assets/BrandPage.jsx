import { PageHeader, Section } from "../../../shared/PagePrimitives.jsx";
import fullLogoUrl from "../../../assets/logos/suncode-logo.svg";
import compactLogoUrl from "../../../assets/logos/suncode-logo-small.svg";

export function BrandPage() {
  return (
    <>
      <PageHeader
        title="Brand marks"
        description="Approved full and compact marks for SunCode product surfaces."
        path="core/assets/brand/"
        status="Asset source"
        tone="review"
      />
      <Section id="brand-marks" title="Brand marks">
        <div className="logo-specimens">
          <div className="logo-on-light">
            <img src={fullLogoUrl} alt="SunCode full logo" />
            <code>suncode-logo.svg</code>
          </div>
          <div className="logo-on-dark">
            <img src={compactLogoUrl} alt="SunCode compact logo" />
            <code>suncode-logo-small.svg</code>
          </div>
        </div>
      </Section>
    </>
  );
}
