export const overviewItem = {
  path: "/",
  label: "Design system",
  icon: "home",
  keywords: "overview index entry architecture layers",
};

export const primaryModules = [
  {
    id: "core",
    label: "Core",
    path: "/core/tokens",
    icon: "foundation",
    items: [
      {
        path: "/core/tokens",
        label: "Tokens",
        icon: "foundation",
        keywords: "colors typography spacing theme",
        children: [
          { id: "colors", label: "Colors", keywords: "color roles semantic theme" },
          { id: "typography", label: "Typography", keywords: "font type hierarchy" },
          { id: "spacing", label: "Spacing & shape", keywords: "spacing radius controls dimensions" },
        ],
      },
      {
        path: "/core/assets",
        label: "Assets",
        icon: "assets",
        keywords: "icons logos fonts",
        children: [
          { id: "brand", label: "Brand marks", keywords: "logo brand" },
          { id: "icons", label: "Interface icons", keywords: "icons svg symbols" },
          { id: "fonts", label: "Fonts", keywords: "font typeface" },
        ],
      },
    ],
  },
  {
    id: "components",
    label: "Components",
    path: "/components/universal",
    icon: "components",
    items: [
      {
        path: "/components/universal",
        label: "Universal",
        icon: "components",
        keywords: "button input checkbox radio toggle badge avatar card modal tooltip markdown",
        children: [
          { path: "/components/universal/actions", label: "Actions", keywords: "button primary danger" },
          { path: "/components/universal/fields", label: "Fields", keywords: "input select textarea validation" },
          { path: "/components/universal/selection", label: "Selection", keywords: "checkbox radio toggle" },
          { path: "/components/universal/surfaces", label: "Surfaces", keywords: "card authority" },
          { path: "/components/universal/overlays", label: "Overlays", keywords: "avatar modal tooltip" },
          { path: "/components/universal/navigation", label: "Navigation", keywords: "tabs segmented filters" },
          { path: "/components/universal/feedback", label: "Feedback", keywords: "status alert progress loading empty" },
          { path: "/components/universal/data", label: "Data", keywords: "code table mono" },
          { path: "/components/universal/markdown", label: "Markdown", keywords: "heading prose lists quote code" },
        ],
      },
      {
        path: "/components/platform-specific",
        label: "Platform-specific",
        icon: "platform",
        keywords: "desktop mobile tui only",
        children: [{ id: "platform-indexes", label: "Platform indexes", keywords: "desktop mobile tui" }],
      },
    ],
  },
  {
    id: "platforms",
    label: "Platforms",
    path: "/platforms/desktop",
    icon: "platform",
    items: [
      {
        path: "/platforms/desktop",
        label: "Desktop",
        icon: "platform",
        keywords: "avalonia shell sidebar inspector",
        children: [
          { id: "anatomy", label: "Window anatomy", keywords: "conversation sidebar review" },
          { id: "ownership", label: "Desktop ownership", keywords: "desktop-only components" },
        ],
      },
      {
        path: "/platforms/mobile",
        label: "Mobile",
        icon: "mobile",
        keywords: "deferred bottom nav tab swipe",
        children: [
          { id: "boundary", label: "Adaptation boundary", keywords: "deferred future" },
          { id: "ownership", label: "Ownership contract", keywords: "tokens components pages" },
        ],
      },
      {
        path: "/platforms/tui",
        label: "TUI",
        icon: "terminal",
        keywords: "deferred terminal command palette tree",
        children: [
          { id: "boundary", label: "Adaptation boundary", keywords: "deferred future" },
          { id: "ownership", label: "Ownership contract", keywords: "tokens components" },
        ],
      },
    ],
  },
  {
    id: "projects",
    label: "Projects",
    path: "/projects/avalonia-desktop",
    icon: "project",
    items: [
      {
        path: "/projects/avalonia-desktop",
        label: "Avalonia Desktop",
        icon: "project",
        keywords: "phase 1 runtime mapping resources",
        children: [
          { id: "runtime-path", label: "Design-to-runtime", keywords: "tokens components mapping" },
          { id: "review-paths", label: "Review paths", keywords: "universal desktop tokens" },
          { id: "runtime-boundary", label: "Runtime boundary", keywords: "avalonia rust react" },
        ],
      },
    ],
  },
];

export function getModuleForPath(path) {
  return primaryModules.find((module) => path.startsWith(`/${module.id}/`));
}

export const allNavigationItems = [
  { ...overviewItem, group: "Overview", moduleId: "overview" },
  ...primaryModules.flatMap((module) =>
    module.items.flatMap((item) => [
      { ...item, group: module.label, moduleId: module.id },
      ...(item.children ?? []).filter((child) => child.path).map((child) => ({ ...child, group: `${module.label} · ${item.label}`, moduleId: module.id })),
    ]),
  ),
];
