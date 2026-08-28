import { useEffect, useMemo, useRef, useState } from "react";
import { UniversalComponentsPage } from "../../components/universal/react/UniversalComponentsPage.jsx";
import { PlatformSpecificPage } from "../../components/platform-specific/react/PlatformSpecificPage.jsx";
import { DesktopPlatformPage } from "../../platforms/desktop/index.jsx";
import { MobilePlatformPage } from "../../platforms/mobile/index.jsx";
import { TuiPlatformPage } from "../../platforms/tui/index.jsx";
import { AvaloniaProjectPage } from "../../projects/avalonia-desktop/index.jsx";
import { AssetsPage } from "./AssetsPage.jsx";
import { Icon } from "./Icon.jsx";
import { navigationGroups, allNavigationItems } from "./navigation.js";
import { NotFoundPage } from "./NotFoundPage.jsx";
import { OverviewPage } from "./OverviewPage.jsx";
import { RouteLink } from "./PagePrimitives.jsx";
import { TokensPage } from "./TokensPage.jsx";
import { useHashRoute } from "./useHashRoute.js";

const routes = {
  "/": OverviewPage,
  "/core/tokens": TokensPage,
  "/core/assets": AssetsPage,
  "/components/universal": UniversalComponentsPage,
  "/components/platform-specific": PlatformSpecificPage,
  "/platforms/desktop": DesktopPlatformPage,
  "/platforms/mobile": MobilePlatformPage,
  "/platforms/tui": TuiPlatformPage,
  "/projects/avalonia-desktop": AvaloniaProjectPage,
};

function readTheme() {
  try { return window.localStorage.getItem("suncode-design-theme") || "light"; }
  catch { return "light"; }
}

export function DesignSystemApp() {
  const path = useHashRoute();
  const [theme, setTheme] = useState(readTheme);
  const [menuOpen, setMenuOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [searchOpen, setSearchOpen] = useState(false);
  const searchRef = useRef(null);
  const Page = routes[path];
  const activeItem = allNavigationItems.find((item) => item.path === path);
  const results = useMemo(() => query.trim() ? allNavigationItems.filter((item) => `${item.label} ${item.group} ${item.keywords}`.toLowerCase().includes(query.trim().toLowerCase())) : [], [query]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    try { window.localStorage.setItem("suncode-design-theme", theme); } catch { /* non-fatal */ }
  }, [theme]);

  useEffect(() => {
    setMenuOpen(false);
    setQuery("");
    setSearchOpen(false);
  }, [path]);

  useEffect(() => {
    if (searchOpen) searchRef.current?.focus();
  }, [searchOpen]);

  useEffect(() => {
    const handleKey = (event) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setSearchOpen(true);
      }
      if (event.key === "Escape") { setQuery(""); setMenuOpen(false); setSearchOpen(false); }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, []);

  return (
    <div className="design-browser">
      <header className="browser-topbar">
        <button className="mobile-menu" aria-label={menuOpen ? "Close navigation" : "Open navigation"} aria-expanded={menuOpen} onClick={() => setMenuOpen(!menuOpen)}><Icon name={menuOpen ? "close" : "menu"} /></button>
        <RouteLink to="/" className="browser-brand"><img src="./core/assets/logos/suncode-logo-small.svg" alt="SunCode" /><strong>SunCode</strong><span>Design System</span></RouteLink>
        <div className="browser-context"><span>{activeItem?.group ?? "Catalog"}</span><strong>{activeItem?.label ?? "Unknown module"}</strong></div>
        <div className="browser-tools">
          <div className={`search-control ${searchOpen ? "is-open" : ""}`}><button className="search-trigger" onClick={() => setSearchOpen(true)} aria-label="Open module search"><Icon name="search" /></button><input ref={searchRef} value={query} onFocus={() => setSearchOpen(true)} onChange={(event) => setQuery(event.target.value)} placeholder="Find a module" aria-label="Find a design system module" /><kbd>⌘K</kbd>{searchOpen && query && <div className="search-results">{results.length ? results.map((item) => <RouteLink to={item.path} key={item.path}><span>{item.group}</span><strong>{item.label}</strong></RouteLink>) : <p>No matching modules</p>}</div>}</div>
          <button className="theme-toggle" onClick={() => setTheme(theme === "light" ? "dark" : "light")} aria-label={`Switch to ${theme === "light" ? "dark" : "light"} theme`}><Icon name={theme === "light" ? "moon" : "sun"} /><span>{theme === "light" ? "Dark" : "Light"}</span></button>
        </div>
      </header>
      <div className="browser-layout">
        <aside className={`browser-sidebar ${menuOpen ? "is-open" : ""}`}>
          <nav aria-label="Design system modules">
            {navigationGroups.map((group) => <div className="nav-group" key={group.label}><span className="nav-group-label">{group.label}</span>{group.items.map((item) => <RouteLink key={item.path} to={item.path} className={`browser-nav-item ${path === item.path ? "active" : ""}`} aria-current={path === item.path ? "page" : undefined}><Icon name={item.icon} /><span>{item.label}</span></RouteLink>)}</div>)}
          </nav>
          <footer><span>Review tooling only</span><code>React · Vite · Hash routes</code></footer>
        </aside>
        {menuOpen && <button className="nav-scrim" aria-label="Close navigation" onClick={() => setMenuOpen(false)} />}
        <main className="browser-main"><div className="browser-content">{Page ? <Page /> : <NotFoundPage path={path} />}</div></main>
      </div>
    </div>
  );
}
