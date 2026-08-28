import { PageHeader, Section } from "../../shared/PagePrimitives.jsx";
import fullLogoUrl from "../../assets/logos/suncode-logo.svg";
import compactLogoUrl from "../../assets/logos/suncode-logo-small.svg";

const icons = ["activity", "arrow-up", "chevron-right", "close", "copy", "files", "git-branch", "lock", "more-horizontal", "panel-left", "panel-right", "pin", "plus", "refresh", "settings"];
const iconModules = import.meta.glob("../../assets/icons/*.svg", { eager: true, query: "?url", import: "default" });
const iconUrls = Object.fromEntries(Object.entries(iconModules).map(([path, url]) => [path.split("/").pop().replace(".svg", ""), url]));

export function AssetsPage() {
  return (
    <>
      <PageHeader title="Core assets" description="Approved brand marks and interface symbols live in one stable catalog before runtime packaging copies them." path="core/assets/" status="Catalog" tone="review" />
      <Section id="brand" title="Brand marks">
        <div className="logo-specimens">
          <div className="logo-on-light"><img src={fullLogoUrl} alt="SunCode full logo" /><code>suncode-logo.svg</code></div>
          <div className="logo-on-dark"><img src={compactLogoUrl} alt="SunCode compact logo" /><code>suncode-logo-small.svg</code></div>
        </div>
      </Section>
      <Section id="icons" title="Interface icons" description="SVG symbols use one restrained visual family; accessible names belong to their controls, not the asset file.">
        <div className="asset-icons">
          {icons.map((name) => <div key={name}><img src={iconUrls[name]} alt="" /><code>{name}</code></div>)}
        </div>
      </Section>
      <Section id="fonts" title="Fonts" description="The review browser uses installed fallbacks; runtime packaging must license and include a font before depending on it.">
        <div className="font-contract"><p><strong>UI</strong><span>Noto Sans · Noto Sans CJK SC · PingFang SC · Helvetica Neue</span></p><p><strong>Code / data</strong><span>JetBrains Mono · SF Mono · Menlo · Consolas</span></p></div>
      </Section>
    </>
  );
}
