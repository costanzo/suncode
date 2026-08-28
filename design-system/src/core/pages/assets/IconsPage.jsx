import { PageHeader, Section } from "../../../shared/PagePrimitives.jsx";

const icons = ["activity", "arrow-up", "chevron-right", "close", "copy", "files", "git-branch", "lock", "more-horizontal", "panel-left", "panel-right", "pin", "plus", "refresh", "settings"];
const iconModules = import.meta.glob("../../../assets/icons/*.svg", { eager: true, query: "?url", import: "default" });
const iconUrls = Object.fromEntries(Object.entries(iconModules).map(([path, url]) => [path.split("/").pop().replace(".svg", ""), url]));

export function IconsPage() {
  return <><PageHeader title="Interface icons" description="A restrained monochrome symbol family for controls and technical context." path="core/assets/icons/" status="Asset source" tone="review" /><Section id="icon-catalog" title="Interface icons" description="Accessible names belong to their controls, not the asset file."><div className="asset-icons">{icons.map((name) => <div key={name}><img src={iconUrls[name]} alt="" /><code>{name}</code></div>)}</div></Section></>;
}
