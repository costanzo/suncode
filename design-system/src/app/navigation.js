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
    path: "/core",
    icon: "foundation",
    items: [
      {
        path: "/core/tokens",
        label: "Tokens",
        icon: "foundation",
        keywords: "colors typography spacing theme",
        children: [
          { path: "/core/tokens/colors", label: "Colors", keywords: "color roles semantic theme" },
          { path: "/core/tokens/typography", label: "Typography", keywords: "font type hierarchy" },
          { path: "/core/tokens/spacing", label: "Spacing & shape", keywords: "spacing radius controls dimensions" },
        ],
      },
      {
        path: "/core/assets",
        label: "Assets",
        icon: "assets",
        keywords: "icons logos fonts",
        children: [
          { path: "/core/assets/brand", label: "Brand marks", keywords: "logo brand" },
          { path: "/core/assets/icons", label: "Interface icons", keywords: "icons svg symbols" },
          { path: "/core/assets/fonts", label: "Fonts", keywords: "font typeface" },
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
          { path: "/components/universal/selection", label: "Selection", keywords: "checkbox radio toggle dropdown menu provider model nested" },
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
        children: [{ path: "/components/platform-specific/platform-indexes", label: "Platform indexes", keywords: "desktop mobile tui" }],
      },
    ],
  },
  {
    id: "platforms",
    label: "Platforms",
    path: "/platforms",
    icon: "platform",
    items: [
      {
        path: "/platforms/desktop",
        label: "Desktop",
        icon: "platform",
        keywords: "avalonia shell sidebar inspector",
        children: [
          { path: "/platforms/desktop/anatomy", label: "Window anatomy", keywords: "conversation sidebar review" },
          { path: "/platforms/desktop/ownership", label: "Desktop ownership", keywords: "desktop-only components" },
        ],
      },
      {
        path: "/platforms/mobile",
        label: "Mobile",
        icon: "mobile",
        keywords: "deferred bottom nav tab swipe",
        children: [
          { path: "/platforms/mobile/boundary", label: "Adaptation boundary", keywords: "deferred future" },
          { path: "/platforms/mobile/ownership", label: "Ownership contract", keywords: "tokens components pages" },
        ],
      },
      {
        path: "/platforms/tui",
        label: "TUI",
        icon: "terminal",
        keywords: "deferred terminal command palette tree",
        children: [
          { path: "/platforms/tui/boundary", label: "Adaptation boundary", keywords: "deferred future" },
          { path: "/platforms/tui/ownership", label: "Ownership contract", keywords: "tokens components" },
        ],
      },
      {
        path: "/platforms/web",
        label: "Web",
        icon: "platform",
        keywords: "deferred browser web client",
        children: [
          { path: "/platforms/web/boundary", label: "Adaptation boundary", keywords: "deferred future browser" },
          { path: "/platforms/web/ownership", label: "Ownership contract", keywords: "tokens components pages" },
        ],
      },
    ],
  },
  {
    id: "projects",
    label: "Projects",
    path: "/projects",
    icon: "project",
    items: [
      {
        path: "/projects/desktop",
        label: "Desktop",
        icon: "project",
        keywords: "project hub avalonia runtime mapping resources",
        children: [
          { path: "/projects/desktop/project-hub", label: "ProjectHub", keywords: "welcome recent projects open project settings" },
          { path: "/projects/desktop/settings", label: "Settings", keywords: "defaults appearance network logging model providers credentials" },
          {
            path: "/projects/desktop/workspace",
            label: "Workspace",
            icon: "workspace",
            keywords: "project workspace sessions explorer conversation review source control provider trace",
            children: [
              { path: "/projects/desktop/workspace/sessions", label: "Sessions", keywords: "session list pin rename archive" },
              { path: "/projects/desktop/workspace/explorer", label: "Explorer", keywords: "project files dependencies tree" },
              { path: "/projects/desktop/workspace/conversation", label: "Conversation", keywords: "messages tools composer model reasoning" },
              { path: "/projects/desktop/workspace/review", label: "Review", keywords: "agent process approvals questions checkpoints undo" },
              { path: "/projects/desktop/workspace/source-control", label: "Source control", keywords: "git changed files diff additions deletions" },
              { path: "/projects/desktop/workspace/provider-trace", label: "Provider trace", keywords: "model calls tokens cache duration request response" },
            ],
          },
        ],
      },
    ],
  },
];

export function getModuleForPath(path) {
  return primaryModules.find((module) => path === `/${module.id}` || path.startsWith(`/${module.id}/`));
}

function flattenItems(items, module, parentLabel = module.label) {
  return items.flatMap((item) => [
    { ...item, group: parentLabel, moduleId: module.id },
    ...flattenItems(item.children ?? [], module, `${parentLabel} · ${item.label}`),
  ]);
}

export const allNavigationItems = [
  { ...overviewItem, group: "Overview", moduleId: "overview" },
  ...primaryModules.flatMap((module) => flattenItems(module.items, module)),
];
