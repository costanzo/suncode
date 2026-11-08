# Official website specimen

Route: `/projects/website` under Projects → Official website. Root `DESIGN.md` owns the specification; `src/styles/tokens/website.css` records the public-site geometry. Shared semantic colors and the approved mark supply the visual identity.

This is the public product information site, separate from Projects → Web (the deferred remote-control client). The overview is a persuasion surface and the feature guide is a reading surface. Both support the shared light paper/slate and dark graphite/silver palettes. The specimen header button switches both samples independently of the catalog shell. Production defaults to the system preference and persists manual choices, applying them before first paint.

The specimen includes wide/constrained layouts, mobile navigation disclosure, the three illustrative workbench stages, four availability-label fixtures, combined search and availability filtering, no-results recovery, and native scope disclosure. Its constrained fixture is 390px wide. It samples composition and states; the actual `website/` routes implement the full workbench, seven-category directory, navigation dismissal, and all 31 `.agents/features/` records. Review those routes alongside this specimen. Do not infer release availability from specimen fixtures.

## Shared desktop preview

`WorkbenchDemo.jsx` is the single interactive window demonstration consumed by this route and `website/app/components/WorkbenchDemo.tsx`. It composes the existing `WorkspaceWindow` with Ubuntu chrome and optional conversation/review/footer fixtures. `workbench.css` imports the component-owned desktop styles and shared tokens, then owns only public-site composition, container adaptation, and motion. The TypeScript declaration describes the narrow public export.

At wide widths, the preview shows Sessions, Conversation, and Review. Below a 1050px content width, navigation yields. At 700px or less, approval Review stacks with Conversation, and the example Git file list stacks above its diff. Inspection and completed-change stages hide the supporting Review at that size. Review in both specimen themes, including a theme opposite to the catalog shell, all three stages, and the 390px constrained fixture. Controls use local sample state. Caption and approval actions report feedback in the preview status region.
