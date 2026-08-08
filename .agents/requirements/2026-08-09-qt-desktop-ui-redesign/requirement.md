# Requirement

## Background

The Phase 1 Qt client exposes the required runtime functions, but its current presentation has weak hierarchy, crowded side panels, generic controls, and empty states that do not guide the user. The primary work path is project/session selection, agent conversation, operation approval, and review or undo of file changes.

## Goals

- Make the conversation workspace the visual and spatial center of the desktop application.
- Let users independently collapse navigation and review panels.
- Show a standalone project hub when no project window is active, and keep opened projects in separate windows.
- Establish restrained, coherent dark and light visual systems appropriate for a focused development tool.
- Improve scanability, empty states, approvals, change review, and runtime status without changing runtime behavior.

## Non-goals

- Runtime, protocol, persistence, or provider changes.
- New product capabilities beyond presentation state for panel visibility.
- CLI, TUI, Web, or mobile surfaces.
- Decorative or high-motion visual effects.

## Requirements

- Preserve all existing Phase 1 desktop functionality.
- Launch into a project list hub instead of auto-opening the first recent project.
- Keep recent projects visible in the hub even when no project is selected.
- Opening a recent or newly chosen project creates a separate project window.
- Closing the last project window returns to the project hub; closing the hub exits the app.
- Provide menu actions for opening another project and opening global settings.
- Keep project/session controls in the left navigation region.
- Keep all sessions for the current project in the left navigation region.
- Keep the conversation view in the central work region.
- Keep agent/process state, approvals, turn changes, files touched, and runtime diagnostics in the right review region.
- Provide global model/provider/API-key settings and a model selector beside the composer.
- Provide accessible controls to collapse and restore the left and right regions.
- Preserve usable focus, disabled, error, empty, and connected states.
- Support the existing minimum window size without overlapping or clipped primary controls.

## Edge cases

- No runtime connection, projects, sessions, messages, approvals, checkpoints, or changed paths.
- Long session identifiers, status messages, paths, approval arguments, and message content.
- Active turn cancellation and unavailable credentials.
- Both side regions collapsed at once.
- One or more recent projects available while no project window is open.
- Multiple project windows opened from the hub or Project menu.

## Acceptance criteria

- The central workspace remains dominant at the default 1440×900 window size.
- Either side region can collapse independently and be restored from persistent top-level controls.
- Existing runtime client calls remain wired to their corresponding controls.
- Recent projects render in the project hub from the runtime project list.
- Project windows do not merge multiple opened projects into tabs.
- The Qt desktop target builds successfully.
- Manual screenshot review finds no material clipping, hierarchy, or contrast defects in the launch state.

## Open questions

- None for this delivery.
