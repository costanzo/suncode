# Changes

## Source

- Avalonia workspace, settings, modal, review, and conversation surfaces.
- Settings now maps the design-system surface hierarchy and spacing directly: 32 px navigation rows, 30 px provider rows, 220 px control columns, 24 px row gaps, 26/28/22 px heading-divider-action rhythm, subordinate certificate trust controls, provider endpoint reset, and discrete model chips.
- The design asset catalog and Avalonia package include the Settings specimen's foundation, sun, platform, assets, and components icons.
- Modal instances use a dedicated dialog-card width while the modal host stretches across the workspace, keeping the overlay coverage aligned with the design-system backdrop.
- The expanded composer modal is hosted at the ProjectWorkspace root and receives draft/submit events through ChatArea and ChatInput, preventing the fixed 126px composer height from clipping the backdrop.
- SCModal uses a three-row header/body/actions layout with an auto-scroll body, a host-relative card max height, and clipped rounded-card contents, so long composer text cannot cover the bottom border or overflow the workspace boundary.
- The expanded composer text field supplies a theme-aware focused border resource and explicit neutral one-pixel focus style, matching the design-system textarea focus state.
- The expanded composer secondary action is labeled `Close` consistently in Avalonia and the design-system specimen.
- SCComboBox no longer mixes a binding and imperative writes for the grouped trigger label; `UpdateGroupedLabel` owns text/foreground updates and runs after both programmatic refreshes and menu selections.
- Certificate trust initialization filters empty persisted paths before creating `PathBuf` values; provider and WebFetch clients only read a custom certificate when system trust is explicitly disabled. This preserves the default system certificate behavior even if an old path is missing.
- Workspace titlebar traffic lights now share the design-system's 1px titlebar inset, aligning the red close control with the gutter below. Copy-response actions switch from the copy glyph to a green check glyph for 1.4 seconds before restoring the copy affordance.
- Copy-response success color is explicitly green per the interaction requirement: Avalonia theme resources expose `CopySuccessSvgCss`, and design-system tokens expose `--copy-success` in both themes.
- Workspace gutters now use the design-system's 26×28 DIP transparent icon buttons, 15px icon glyphs, six-pixel grouping gaps, and active surface/border treatment. Pointer hover keeps the button background transparent and strengthens the icon stroke instead of lifting the whole button background.
- ProjectHub top actions now use an explicit 30px centered action style and the design-system's 8px action gap, keeping Settings and Open project text optically centered in the 62px toolbar.
- ProjectHub action buttons retain the design-system's 14px horizontal button padding so the Open project label and icon have enough measured width and cannot be clipped at the trailing edge.
- Conversation composer now follows the design-system compact layout: 24px side margins, a 52px minimum textarea, explicit 24px inner controls, correctly separated left actions, and the matching “Ask SunCode to work on this project” placeholder and expand glyph.
- Composer model selection now keeps the selected label in a bindable control property after grouped-menu selection and resolves the selected item back to the ViewModel-owned model. The grouped control also rebinds when it enters the visual tree, preventing an initially blank selector.
- Attachment capability metadata for the seeded DeepSeek V4 Flash and GPT 5.6 SOL models is enabled in the current SQLite seed/update script, allowing the attachment action when either configured vision-capable model is selected.

## Tests

- Desktop build and focused unit tests.
- Design-system production build.
