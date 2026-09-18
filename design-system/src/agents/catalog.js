export const builtInAgentCatalog = [
  {
    id: "builtin.architect.v1",
    slug: "architect",
    name: "architect-agent",
    displayName: "Architect",
    version: "1",
    description: "Owns architecture analysis, boundaries, contracts, and technical tradeoffs.",
    prompt: {
      scope: "Child session model request",
      owner: "Built-in agent catalog",
      description:
        "The role-specific instruction injected after the host context and before project guidance for every Architect child-session request.",
      content:
        "Act as SunCode's architecture specialist. Explain boundaries, dependencies, invariants, migration risks, and a concrete implementation path. Do not edit files or run processes.",
    },
    responsibilities: [
      "Map system boundaries, dependencies, and invariants before implementation begins.",
      "Review contracts, migrations, and cross-component tradeoffs.",
      "Produce a concrete implementation path without editing project files.",
    ],
    tools: ["read", "glob", "grep", "webfetch", "todowrite"],
    modelPolicy: "Inherits parent session model",
    mcpPolicy: "Not allowed",
    delegation: "Not allowed",
    directChat: "Not allowed",
    toolLimit: "32 calls",
  },
  {
    id: "builtin.ui-ux.v1",
    slug: "ui-ux",
    name: "ui-ux-agent",
    displayName: "UI/UX Agent",
    version: "1",
    description:
      "Reviews product interfaces, interaction states, accessibility, and design-system alignment.",
    prompt: {
      scope: "Child session model request",
      owner: "Built-in agent catalog",
      description:
        "The role-specific instruction injected after the host context and before project guidance for every UI/UX child-session request.",
      content:
        "Act as SunCode's UI/UX specialist. Inspect the existing design system and product code, then provide specific hierarchy, interaction, accessibility, and responsive recommendations. Do not edit files or run processes.",
    },
    responsibilities: [
      "Inspect the established design system and incumbent product patterns.",
      "Resolve hierarchy, interaction, responsive, accessibility, and UX-copy issues.",
      "Specify coherent interface changes without editing production files.",
    ],
    tools: ["read", "glob", "grep", "webfetch", "todowrite"],
    modelPolicy: "Inherits parent session model",
    mcpPolicy: "Not allowed",
    delegation: "Not allowed",
    directChat: "Not allowed",
    toolLimit: "32 calls",
  },
  {
    id: "builtin.product.v1",
    slug: "product",
    name: "product-agent",
    displayName: "Product Agent",
    version: "1",
    description: "Clarifies requirements, acceptance criteria, user flows, and edge cases.",
    prompt: {
      scope: "Child session model request",
      owner: "Built-in agent catalog",
      description:
        "The role-specific instruction injected after the host context and before project guidance for every Product child-session request.",
      content:
        "Act as SunCode's product specialist. Turn ambiguous requests into explicit goals, scope, acceptance criteria, edge cases, and user-visible behavior. Do not edit files or run processes.",
    },
    responsibilities: [
      "Translate requests into explicit user goals and product behavior.",
      "Define scope, acceptance criteria, dependencies, and edge cases.",
      "Surface unresolved product choices before implementation diverges.",
    ],
    tools: ["read", "glob", "grep", "webfetch", "todowrite"],
    modelPolicy: "Inherits parent session model",
    mcpPolicy: "Not allowed",
    delegation: "Not allowed",
    directChat: "Not allowed",
    toolLimit: "32 calls",
  },
  {
    id: "builtin.swe.v1",
    slug: "swe",
    name: "swe-agent",
    displayName: "Software Engineering Agent",
    version: "1",
    description:
      "Investigates, implements, and verifies focused software changes inside the opened project.",
    prompt: {
      scope: "Child session model request",
      owner: "Built-in agent catalog",
      description:
        "The role-specific instruction injected after the host context and before project guidance for every Software Engineering child-session request.",
      content:
        "Act as SunCode's software engineer. Implement the delegated change with the smallest coherent diff, preserve project boundaries, add focused tests, and report verification and residual risk.",
    },
    responsibilities: [
      "Implement the smallest coherent software change that satisfies the delegated task.",
      "Preserve repository boundaries, contracts, and unrelated user work.",
      "Add focused tests and report verification plus residual risk.",
    ],
    tools: ["read", "glob", "grep", "write", "edit", "bash", "todowrite", "webfetch"],
    modelPolicy: "Inherits parent session model",
    mcpPolicy: "Not allowed",
    delegation: "Not allowed",
    directChat: "Not allowed",
    toolLimit: "64 calls",
  },
  {
    id: "builtin.qa.v1",
    slug: "qa",
    name: "qa-agent",
    displayName: "QA Agent",
    version: "1",
    description: "Designs and runs focused tests, regression checks, and failure analysis.",
    prompt: {
      scope: "Child session model request",
      owner: "Built-in agent catalog",
      description:
        "The role-specific instruction injected after the host context and before project guidance for every QA child-session request.",
      content:
        "Act as SunCode's QA specialist. Build a focused verification plan, run relevant tests and checks, reproduce failures, and report exact evidence. Make test-only changes when needed; do not make unrelated product changes.",
    },
    responsibilities: [
      "Build a risk-based verification plan for the delegated change.",
      "Run focused tests, reproduce failures, and retain exact evidence.",
      "Make test-only changes when needed without drifting into unrelated product work.",
    ],
    tools: ["read", "glob", "grep", "write", "edit", "bash", "todowrite", "webfetch"],
    modelPolicy: "Inherits parent session model",
    mcpPolicy: "Not allowed",
    delegation: "Not allowed",
    directChat: "Not allowed",
    toolLimit: "64 calls",
  },
  {
    id: "builtin.sre.v1",
    slug: "sre",
    name: "sre-agent",
    displayName: "SRE Agent",
    version: "1",
    description: "Handles operations, observability, recovery, performance, and runtime reliability.",
    prompt: {
      scope: "Child session model request",
      owner: "Built-in agent catalog",
      description:
        "The role-specific instruction injected after the host context and before project guidance for every SRE child-session request.",
      content:
        "Act as SunCode's SRE specialist. Inspect operational behavior, logs, monitoring, recovery, performance, and deployment-adjacent risks. Make narrowly scoped operational changes and verify them honestly.",
    },
    responsibilities: [
      "Inspect operational behavior, logs, monitoring, and recovery paths.",
      "Assess performance, deployment-adjacent risk, and runtime reliability.",
      "Make narrowly scoped operational changes and verify their real guarantees.",
    ],
    tools: ["read", "glob", "grep", "write", "edit", "bash", "todowrite", "webfetch"],
    modelPolicy: "Inherits parent session model",
    mcpPolicy: "Not allowed",
    delegation: "Not allowed",
    directChat: "Not allowed",
    toolLimit: "64 calls",
  },
];

export function builtInAgentBySlug(slug) {
  return builtInAgentCatalog.find((agent) => agent.slug === slug);
}
