# Requirement

## Background

The desktop conversation review page already shows the compact composer, message history, tool rows, and tool-detail modal. Three important conversation behaviors need explicit executable design coverage: an expanded drafting mode for long prompts, a live command-output view for long-running tool execution, and a dedicated thinking state that differs visually from generic running work.

## Goals

- Add an expanded composer interaction to the design-system conversation surface.
- Add a live tool-output inspection interaction to the design-system conversation surface.
- Add a distinct thinking state to the design-system conversation surface.
- Keep the implementation limited to the React design review browser and design authority documents.

## Non-goals

- Change the Avalonia desktop client.
- Change the Rust agent, SDK contracts, persistence, or tool execution behavior.
- Introduce production web runtime behavior.

## Requirements

- The compact conversation composer must expose an explicit expand affordance.
- Expanding the composer must open a modal with a large drafting textarea suitable for long-form input.
- The expanded composer modal must show a live character count in the lower-right area of the drafting surface.
- The conversation review page must include a dedicated specimen state showing the expanded-composer presentation.
- Long-running tool execution must support opening a modal that includes a live command-output region in addition to request and status details.
- The conversation review page must include a dedicated specimen state showing the live tool-output modal while work is still running.
- The conversation review page must include a dedicated specimen state showing the assistant in a thinking phase.
- The thinking phase must not show the three-dot running indicator and must instead animate the word `Thinking` from left to right in a repeating reveal.
- The implementation must reuse the existing design-system modal language, graphite/silver visual identity, and semantic state colors.

## Edge cases

- Character counts must update continuously for multi-paragraph input.
- The expanded composer must remain useful when attachments already exist in the compact composer.
- Live output must remain readable for long lines and multi-line command streams.
- The thinking animation must remain legible at compact widths and must not read like an error or approval state.
- Compact review pages must continue to degrade gracefully on narrower widths.

## Acceptance criteria

- The conversation review page shows a visible expand control in the composer.
- Opening the expanded composer displays a larger textarea and live character count.
- Running tool rows can present a modal with streaming command output.
- The conversation review page includes a dedicated thinking specimen that replaces the three-dot indicator with animated `Thinking` text.
- The design-system build succeeds.
- `git diff --check` succeeds.

## Open questions

- None.
