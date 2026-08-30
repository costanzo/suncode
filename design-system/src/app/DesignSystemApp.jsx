import { useEffect, useRef, useState } from "react";
import { UniversalComponentsPage } from "../components/universal/UniversalComponentsPage.jsx";
import { ActionsPage } from "../components/universal/modules/actions/index.js";
import { DataPage } from "../components/universal/modules/data/index.js";
import { FeedbackPage } from "../components/universal/modules/feedback/index.js";
import { FieldsPage } from "../components/universal/modules/fields/index.js";
import { MarkdownPage } from "../components/universal/modules/markdown/index.js";
import { NavigationPage } from "../components/universal/modules/navigation/index.js";
import { OverlaysPage } from "../components/universal/modules/overlays/index.js";
import { SelectionPage } from "../components/universal/modules/selection/index.js";
import { SurfacesPage } from "../components/universal/modules/surfaces/index.js";
import { PlatformSpecificPage } from "../components/platform-specific/PlatformSpecificPage.jsx";
import { PlatformIndexesPage } from "../components/platform-specific/PlatformIndexesPage.jsx";
import { AssetsPage } from "../core/pages/AssetsPage.jsx";
import { CorePage } from "../core/pages/CorePage.jsx";
import { BrandPage } from "../core/pages/assets/BrandPage.jsx";
import { FontsPage } from "../core/pages/assets/FontsPage.jsx";
import { IconsPage } from "../core/pages/assets/IconsPage.jsx";
import { NotFoundPage } from "../core/pages/NotFoundPage.jsx";
import { OverviewPage } from "../core/pages/OverviewPage.jsx";
import { TokensPage } from "../core/pages/TokensPage.jsx";
import { ColorsPage } from "../core/pages/tokens/ColorsPage.jsx";
import { SpacingPage } from "../core/pages/tokens/SpacingPage.jsx";
import { TypographyPage } from "../core/pages/tokens/TypographyPage.jsx";
import { DesktopPlatformPage } from "../platforms/desktop/index.jsx";
import { DesktopAnatomyPage } from "../platforms/desktop/AnatomyPage.jsx";
import { DesktopOwnershipPage } from "../platforms/desktop/OwnershipPage.jsx";
import { DeferredBoundaryPage, DeferredOwnershipPage } from "../platforms/DeferredPages.jsx";
import { PlatformsPage } from "../platforms/PlatformsPage.jsx";
import { MobilePlatformPage } from "../platforms/mobile/index.jsx";
import { TuiPlatformPage } from "../platforms/tui/index.jsx";
import { WebPlatformPage } from "../platforms/web/index.jsx";
import { ProjectsPage } from "../projects/ProjectsPage.jsx";
import { ProjectHubPage } from "../projects/desktop/index.jsx";
import { AboutPage } from "../projects/desktop/about/index.jsx";
import { DesktopProjectPage } from "../projects/desktop/DesktopProjectPage.jsx";
import { SettingsPage } from "../projects/desktop/settings/index.jsx";
import { WorkspacePage } from "../projects/desktop/workspace/index.jsx";
import { WorkspaceSessionsPage } from "../projects/desktop/workspace/sessions/index.jsx";
import { WorkspaceExplorerPage } from "../projects/desktop/workspace/explorer/index.jsx";
import { WorkspaceConversationPage } from "../projects/desktop/workspace/conversation/index.jsx";
import { WorkspaceReviewPage } from "../projects/desktop/workspace/review/index.jsx";
import { WorkspaceSourceControlPage } from "../projects/desktop/workspace/source-control/index.jsx";
import { WorkspaceProviderTracePage } from "../projects/desktop/workspace/provider-trace/index.jsx";
import { Icon } from "../shared/Icon.jsx";
import { RouteLink } from "../shared/PagePrimitives.jsx";
import compactLogoUrl from "../assets/logos/suncode-logo-small.svg";
import { getModuleForPath, overviewItem, primaryModules } from "./navigation.js";
import { useHashRoute } from "./useHashRoute.js";

const routes = {
  "/": OverviewPage,
  "/core": CorePage,
  "/core/tokens": TokensPage,
  "/core/tokens/colors": ColorsPage,
  "/core/tokens/typography": TypographyPage,
  "/core/tokens/spacing": SpacingPage,
  "/core/assets": AssetsPage,
  "/core/assets/brand": BrandPage,
  "/core/assets/icons": IconsPage,
  "/core/assets/fonts": FontsPage,
  "/components/universal": UniversalComponentsPage,
  "/components/universal/actions": ActionsPage,
  "/components/universal/fields": FieldsPage,
  "/components/universal/selection": SelectionPage,
  "/components/universal/surfaces": SurfacesPage,
  "/components/universal/overlays": OverlaysPage,
  "/components/universal/navigation": NavigationPage,
  "/components/universal/feedback": FeedbackPage,
  "/components/universal/data": DataPage,
  "/components/universal/markdown": MarkdownPage,
  "/components/platform-specific": PlatformSpecificPage,
  "/components/platform-specific/platform-indexes": PlatformIndexesPage,
  "/platforms": PlatformsPage,
  "/platforms/desktop": DesktopPlatformPage,
  "/platforms/desktop/anatomy": DesktopAnatomyPage,
  "/platforms/desktop/ownership": DesktopOwnershipPage,
  "/platforms/mobile": MobilePlatformPage,
  "/platforms/mobile/boundary": () => <DeferredBoundaryPage platform="mobile" title="Mobile" icon="mobile" />,
  "/platforms/mobile/ownership": () => <DeferredOwnershipPage platform="mobile" title="Mobile" />,
  "/platforms/tui": TuiPlatformPage,
  "/platforms/tui/boundary": () => <DeferredBoundaryPage platform="tui" title="TUI" icon="terminal" />,
  "/platforms/tui/ownership": () => <DeferredOwnershipPage platform="tui" title="TUI" />,
  "/platforms/web": () => <WebPlatformPage />,
  "/platforms/web/boundary": () => <DeferredBoundaryPage platform="web" title="Web" icon="platform" />,
  "/platforms/web/ownership": () => <DeferredOwnershipPage platform="web" title="Web" />,
  "/projects": ProjectsPage,
  "/projects/desktop": DesktopProjectPage,
  "/projects/desktop/project-hub": ProjectHubPage,
  "/projects/desktop/about": AboutPage,
  "/projects/desktop/settings": SettingsPage,
  "/projects/desktop/workspace": WorkspacePage,
  "/projects/desktop/workspace/sessions": WorkspaceSessionsPage,
  "/projects/desktop/workspace/explorer": WorkspaceExplorerPage,
  "/projects/desktop/workspace/conversation": WorkspaceConversationPage,
  "/projects/desktop/workspace/review": WorkspaceReviewPage,
  "/projects/desktop/workspace/source-control": WorkspaceSourceControlPage,
  "/projects/desktop/workspace/provider-trace": WorkspaceProviderTracePage,
};

function readTheme() {
  try { return window.localStorage.getItem("suncode-design-theme") || "light"; }
  catch { return "light"; }
}

function readCompactLayout() {
  return window.matchMedia("(max-width: 1080px)").matches;
}

function readSidebarCollapsed() {
  try { return window.localStorage.getItem("suncode-design-sidebar-collapsed") === "true"; }
  catch { return false; }
}

function SidebarTreeItem({ item, path, expandedItems, onToggle, onRouteNavigate, level = 0, collapsed = false }) {
  const hasChildren = item.children?.length > 0;
  const itemIsCurrent = path === item.path || path.startsWith(`${item.path}/`);
  const expanded = hasChildren && (expandedItems[item.path] ?? itemIsCurrent);
  const controlId = `nav-children-${item.path.replace(/[^a-z0-9]+/gi, "-")}`;

  return (
    <div className={`sidebar-tree-item level-${level}`}>
      <div className={`sidebar-nav-row ${itemIsCurrent ? "is-current" : ""}`}>
        <RouteLink to={item.path} className={`browser-nav-item browser-nav-link ${path === item.path ? "active" : ""}`} aria-label={collapsed ? item.label : undefined} title={collapsed ? item.label : undefined} aria-current={path === item.path ? "page" : undefined} aria-expanded={hasChildren && !collapsed ? expanded : undefined} aria-controls={hasChildren && !collapsed ? controlId : undefined} onClick={() => { if (hasChildren) onToggle(item); onRouteNavigate(); }}>
          {level === 0 && <Icon name={item.icon} />}
          <span>{item.label}</span>
          {hasChildren && <Icon name="chevron-right" className={`sidebar-nav-chevron ${expanded ? "is-expanded" : ""}`} size={15} />}
        </RouteLink>
      </div>
      {hasChildren && expanded && (
        <div className="sidebar-nav-children" id={controlId}>
          {item.children.map((child) => child.children?.length ? (
            <SidebarTreeItem key={child.path} item={child} path={path} expandedItems={expandedItems} onToggle={onToggle} onRouteNavigate={onRouteNavigate} level={level + 1} collapsed={collapsed} />
          ) : (
            <RouteLink key={child.path} to={child.path} className={`browser-nav-item browser-nav-section level-${level + 1} ${path === child.path ? "active" : ""}`} aria-current={path === child.path ? "page" : undefined} onClick={onRouteNavigate}>
              <span>{child.label}</span>
            </RouteLink>
          ))}
        </div>
      )}
    </div>
  );
}

export function DesignSystemApp() {
  const path = useHashRoute();
  const [theme, setTheme] = useState(readTheme);
  const [menuOpen, setMenuOpen] = useState(false);
  const [moduleMenuOpen, setModuleMenuOpen] = useState(false);
  const [expandedItems, setExpandedItems] = useState({});
  const [compactLayout, setCompactLayout] = useState(readCompactLayout);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(readSidebarCollapsed);
  const sidebarRef = useRef(null);
  const mobileMenuRef = useRef(null);
  const priorMenuOpenRef = useRef(false);
  const Page = routes[path];
  const activeModule = getModuleForPath(path);
  const sidebarItems = activeModule?.items ?? [overviewItem];

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    try { window.localStorage.setItem("suncode-design-theme", theme); } catch { /* non-fatal */ }
  }, [theme]);

  useEffect(() => {
    try { window.localStorage.setItem("suncode-design-sidebar-collapsed", String(sidebarCollapsed)); } catch { /* non-fatal */ }
  }, [sidebarCollapsed]);

  useEffect(() => {
    const media = window.matchMedia("(max-width: 1080px)");
    const handleChange = (event) => setCompactLayout(event.matches);
    media.addEventListener("change", handleChange);
    return () => media.removeEventListener("change", handleChange);
  }, []);

  useEffect(() => {
    if (!compactLayout) {
      priorMenuOpenRef.current = menuOpen;
      return undefined;
    }
    const frame = window.requestAnimationFrame(() => {
      if (menuOpen) sidebarRef.current?.querySelector("button, a")?.focus();
      else if (priorMenuOpenRef.current) mobileMenuRef.current?.focus();
      priorMenuOpenRef.current = menuOpen;
    });
    return () => window.cancelAnimationFrame(frame);
  }, [compactLayout, menuOpen]);

  useEffect(() => {
    setMenuOpen(false);
    setModuleMenuOpen(false);
  }, [path]);

  useEffect(() => {
    const handleKey = (event) => {
      if (event.key === "Escape") { setMenuOpen(false); setModuleMenuOpen(false); }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, []);

  const toggleSidebarItem = (item) => {
    const itemIsCurrent = path === item.path || path.startsWith(`${item.path}/`);
    setExpandedItems((current) => ({ ...current, [item.path]: !(current[item.path] ?? itemIsCurrent) }));
  };

  const handleRouteNavigate = () => setMenuOpen(false);

  return (
    <div className={`design-browser ${sidebarCollapsed && !compactLayout ? "is-sidebar-collapsed" : ""}`}>
      <header className="browser-topbar">
        <button ref={mobileMenuRef} className="mobile-menu" aria-label={menuOpen ? "Close submodule navigation" : "Open submodule navigation"} aria-expanded={menuOpen} onClick={() => { setMenuOpen(!menuOpen); setModuleMenuOpen(false); }}><Icon name={menuOpen ? "close" : "menu"} /></button>
        <RouteLink to="/" className="browser-brand"><img src={compactLogoUrl} alt="SunCode" /><strong>SunCode</strong><span>Design System</span></RouteLink>
        <div className="browser-tools">
          <button className="theme-toggle" onClick={() => setTheme(theme === "light" ? "dark" : "light")} aria-label={`Switch to ${theme === "light" ? "dark" : "light"} theme`}><Icon name={theme === "light" ? "moon" : "sun"} /><span>{theme === "light" ? "Dark" : "Light"}</span></button>
        </div>
        <nav className={`primary-module-nav ${moduleMenuOpen ? "is-open" : ""}`} aria-label="Primary design system modules">
          {primaryModules.map((module) => <RouteLink key={module.id} to={module.path} className={`primary-module-link ${activeModule?.id === module.id ? "active" : ""}`} aria-current={activeModule?.id === module.id ? "page" : undefined}><span>{module.label}</span></RouteLink>)}
        </nav>
        <button className="module-menu-trigger" aria-label={moduleMenuOpen ? "Close primary modules" : "Open primary modules"} aria-expanded={moduleMenuOpen} onClick={() => { setModuleMenuOpen(!moduleMenuOpen); setMenuOpen(false); }}><Icon name="components" /><span>{activeModule?.label ?? "Modules"}</span></button>
      </header>
        {moduleMenuOpen && <button className="module-nav-scrim" aria-label="Close primary modules" onClick={() => setModuleMenuOpen(false)} />}
      <div className="browser-layout">
        <aside ref={sidebarRef} className={`browser-sidebar ${menuOpen ? "is-open" : ""}`} aria-hidden={compactLayout && !menuOpen ? "true" : undefined} inert={compactLayout && !menuOpen}>
          <nav aria-label={`${activeModule?.label ?? "Overview"} submodules`}>
            <div className="nav-group">
              <div className="nav-group-heading">
                <span className="nav-group-label">{activeModule ? "Submodules" : "Start here"}</span>
                <button type="button" className="sidebar-collapse-toggle" aria-label={sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"} title={sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"} aria-expanded={!sidebarCollapsed} onClick={() => setSidebarCollapsed((collapsed) => !collapsed)}><Icon name="panel-left" size={15} /></button>
              </div>
              {sidebarItems.map((item) => <SidebarTreeItem key={item.path} item={item} path={path} expandedItems={expandedItems} onToggle={toggleSidebarItem} onRouteNavigate={handleRouteNavigate} collapsed={sidebarCollapsed && !compactLayout} />)}
            </div>
          </nav>
        </aside>
        {menuOpen && <button className="nav-scrim" aria-label="Close navigation" onClick={() => setMenuOpen(false)} />}
        <main className="browser-main"><div className="browser-content">{Page ? <Page /> : <NotFoundPage path={path} />}</div></main>
      </div>
    </div>
  );
}
