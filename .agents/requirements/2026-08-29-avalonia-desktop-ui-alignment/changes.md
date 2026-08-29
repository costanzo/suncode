# Changes

## Source

- Update `apps/desktop-avalonia/App.axaml` shared theme resources and control styles.
- Refine the custom chrome and presentation XAML under `apps/desktop-avalonia/Views/` in the staged order.
- Change C# only when required to preserve or expose existing responsive presentation state; do not add new SDK ownership.

## Contracts and generated artifacts

- No protocol contracts or generated artifacts change.
- The React prototypes remain review inputs and are not generated into Avalonia.

## Configuration and persistence

- No configuration schema or persistence changes are planned.
- Composer attachments are transient local preview state and are not persisted or included in the submit-turn payload.
- Existing theme selection continues to drive Avalonia theme resources.

## Tests

- Run the desktop build after each coherent XAML stage.
- Run focused desktop tests after shared resources and after each behavior-adjacent stage.
- Add tests only when presentation-supporting C# behavior changes.
- Perform manual dark/light, resizing, focus, overflow, empty-state, and dialog checks.

## Documentation

- Maintain this package while delivery is active.
- Update `features/avalonia-desktop-phase-1/` with stable visual implementation facts at closeout.
