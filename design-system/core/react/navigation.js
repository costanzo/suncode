export const navigationGroups = [
  { label: "Overview", items: [{ path: "/", label: "Design system", icon: "home", keywords: "overview index entry" }] },
  { label: "Core", items: [
    { path: "/core/tokens", label: "Tokens", icon: "foundation", keywords: "colors typography spacing theme" },
    { path: "/core/assets", label: "Assets", icon: "assets", keywords: "icons logos fonts" },
  ] },
  { label: "Components", items: [
    { path: "/components/universal", label: "Universal", icon: "components", keywords: "button input checkbox radio toggle badge avatar card modal tooltip markdown" },
    { path: "/components/platform-specific", label: "Platform-specific", icon: "platform", keywords: "desktop mobile tui only" },
  ] },
  { label: "Platforms", items: [
    { path: "/platforms/desktop", label: "Desktop", icon: "platform", keywords: "avalonia shell sidebar inspector" },
    { path: "/platforms/mobile", label: "Mobile", icon: "mobile", keywords: "deferred bottom nav tab swipe" },
    { path: "/platforms/tui", label: "TUI", icon: "terminal", keywords: "deferred terminal command palette tree" },
  ] },
  { label: "Projects", items: [
    { path: "/projects/avalonia-desktop", label: "Avalonia Desktop", icon: "project", keywords: "phase 1 runtime mapping resources" },
  ] },
];

export const allNavigationItems = navigationGroups.flatMap((group) =>
  group.items.map((item) => ({ ...item, group: group.label })),
);
