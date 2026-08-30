import { PageHeader, Section } from "../../shared/PagePrimitives.jsx";

export function DesktopOwnershipPage() {
  return <><PageHeader title="Desktop ownership" description="Components that belong specifically to the desktop adaptation layer." path="platforms/desktop/ownership/" /><Section id="desktop-components" title="Desktop-only ownership"><div className="measurement-grid"><div><code>titlebar/</code><strong>Native window frame</strong></div><div><code>sidebar/</code><strong>Navigation bay</strong></div><div><code>review-inspector/</code><strong>Authority + changes</strong></div><div><code>dropdown-menu/</code><strong>Native menu behavior</strong></div><div><code>data-table/</code><strong>Dense project data</strong></div></div></Section></>;
}
