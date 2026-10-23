import { PageHeader, Section } from "../../../shared/PagePrimitives.jsx";
import { BrandMark } from "../../../shared/BrandMark.jsx";
import fullLogoUrl from "../../../assets/logos/suncode-logo.svg";
import macLogoUrl from "../../../assets/platform/suncode-macos.svg";
import "../styles/brand.css";

export function BrandPage() {
  return (
    <>
      <PageHeader
        title="Brand marks"
        description="The official SunCode mark, optical sizes, and application icon across both themes."
        path="core/assets/brand/"
        status="Asset source"
        tone="review"
      />
      <Section
        id="brand-marks"
        title="In-app marks"
        description="Use the transparent mark in headers and navigation. The 16px and 24px marks retain the optical adjustments from the reference artwork."
      >
        <div className="logo-specimens">
          {["light", "dark"].map((theme) => (
            <div key={theme} className={`logo-on-${theme} brand-preview-${theme}`}>
              <div className="brand-mark-sizes">
                {[40, 24, 16].map((size) => (
                  <figure key={size}>
                    <BrandMark size={size} />
                    <figcaption>
                      <code>{size}px</code>
                    </figcaption>
                  </figure>
                ))}
              </div>
              <code>{theme === "light" ? "#2C3742" : "#D9E0E6"} · transparent</code>
            </div>
          ))}
        </div>
      </Section>
      <Section
        id="application-icons"
        title="Application icon"
        description="The graphite plate and silver mark remain consistent across themes. Native platforms own masking and safe areas."
      >
        <div className="logo-specimens">
          {["light", "dark"].map((theme) => (
            <div key={theme} className={`logo-on-${theme}`}>
              <img
                className="brand-application-preview"
                src={fullLogoUrl}
                alt="SunCode application icon"
              />
              <code>suncode-logo.svg</code>
            </div>
          ))}
        </div>
        <p className="brand-usage-note">
          Keep the mark square and undistorted. Do not add a gradient, outline, or a second
          container behind an in-app mark. Android uses a centered adaptive foreground; iOS receives
          a full-bleed opaque square. The favicon uses the 16px geometry and follows the browser
          theme.
        </p>
      </Section>
      <Section
        id="macos-dock-icons"
        title="macOS Dock icon"
        description="An 824px plate is centered on a transparent 1024px canvas. The live Dock PNG and packaged ICNS use the same inset and preserve the artwork's proportions."
      >
        <div className="logo-specimens">
          {["light", "dark"].map((theme) => (
            <div key={theme} className={`logo-on-${theme}`}>
              <img
                className="brand-application-preview"
                src={macLogoUrl}
                alt="SunCode macOS icon with Dock safe-area padding"
              />
              <code>macOS · 100px transparent inset</code>
            </div>
          ))}
        </div>
      </Section>
    </>
  );
}
