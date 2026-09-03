# Progress

- Status: Complete with contract gaps documented
- Last updated: 2026-09-03

## Completed

- Static comparison of design-system and Avalonia surfaces.
- Confirmation, composer, conversation state, review state, geometry, settings presentation, and tool detail alignment implemented.
- Settings presentation was re-audited against the executable specimen and fully aligned across Defaults, Appearance, Network, Logging, provider overview, and provider detail pages. The stale navigation groups, canvas-backed content surface, oversized rows, and ad hoc form layouts were removed.
- Provider settings now include the specified reset-to-seeded-endpoint action and render available models as individual compact code surfaces.
- Button focus treatment now uses keyboard-only `:focus-visible`, so pointer clicks do not leave a dark focus border while keyboard navigation remains visible.
- Provider detail URL and credential fields explicitly retain their one-pixel resting border after pointer focus, matching the design specimen; provider navigation clicks likewise do not add a focus outline.
- Provider overview/detail rendering now has one owning scroll surface, preventing nested measurement from collapsing the provider list to its minimum content width.
- Provider overview item containers now stretch their generated `ContentPresenter`; this fixes the remaining per-row minimum-content sizing visible in the native Settings window.
- SCModal now raises the control itself above Workspace sibling panels so its backdrop covers the full client area, including the Review bay.
- SCModal now separates its full-size overlay host from the dialog card width via `DialogWidth`, so fixed-size cards no longer constrain the backdrop to a narrow center strip.
- Expanded Composer now reuses the global SCModal from the ProjectWorkspace host instead of the fixed-height ChatInput, allowing its backdrop to cover the complete workspace/window.
- SCModal now constrains its dialog card to the available host height, clips child content to the rounded card, and scrolls oversized content inside the body, keeping all borders and actions within the viewport.
- Expanded Composer's focused text field now overrides the default Avalonia focus resource and keeps the design-system's one-pixel neutral border instead of turning into a dark two-pixel outline.
- Expanded Composer's secondary action now uses the shorter `Close` label in both production and design-system surfaces.
- Grouped model selector now updates its trigger label through one centralized display path after menu selection and selector refresh, so the chosen model remains visible.
- Rust HTTPS configuration now treats the default empty `certificate_path` JSON string as unset and ignores stale custom paths while system certificates are enabled, preventing retries from failing on missing certificate files.
- Workspace traffic-light alignment now uses the design-system's titlebar offset, and response copy controls show a green check icon for the success feedback interval in both Avalonia and the design-system specimen.
- Copy feedback uses dedicated theme-aware green action tokens (`CopySuccessSvgCss` / `--copy-success`) so it remains visibly green even though the general semantic success token is intentionally blue-gray.
- Workspace gutter buttons and icon assets now match the design-system dimensions, icon geometry, active state, and icon-only hover emphasis.
- ProjectHub Settings/Open project controls now have explicit centered content alignment and toolbar spacing matching the design specimen.
- ProjectHub action padding now matches the shared button token, preventing horizontal clipping of the Open project label.
- Conversation composer sizing, spacing, placeholder copy, dropdown controls, and footer action layout now match the design-system specimen.
- Composer attachment and model selection behavior is repaired: vision-capable seeded models enable the attachment picker, and grouped model selections remain visible and synchronized with the ViewModel.
- Desktop build, focused tests, design-system build, and diff checks pass.

## In progress

## Completed contract extensions

- Certificate trust source/path is persisted and applied by Rust Provider and WebFetch clients.
- Bash operations emit bounded live `tool.output` events and Avalonia renders a scrolling live-output panel.
- `context.compacted` is projected into `session_call` as an internal Provider Trace exchange.
- `retry_last_turn` is exposed through Rust SDK/C ABI and invoked by the Review panel.
