# Requirement

## Background

Windows desktop publishing currently requires a bundled PowerShell 7 runtime. Agent commands and the interactive desktop terminal need independently configurable shells.

## Goals

Remove bundled PowerShell, default Windows to system Windows PowerShell, expose two global shell preferences, verify configured executables, and offer user-initiated PowerShell 7 ZIP installation on Windows.

## Current delivery stage

The user requested the design-system Settings page first. This stage implements only the interactive review specimen and its design specification. Production settings, installer, execution, and packaging changes remain pending.

## Requirements

- Settings → Terminal has independent Agent shell and Terminal shell choices, custom paths, automatic detection, manual retry, and Save.
- Windows defaults to Windows PowerShell 5.1. Display “For the best experience, use PowerShell 7.” with an installation action.
- Select Windows x64/ARM64 assets; install into a managed user directory without administrator access or system PATH changes.
- A visible installation preference controls whether successful installation switches both shell configurations.
- Retain prior configurations on invalid drafts or installation failure/cancellation.
- New Agent turns and new/restarted terminal tabs use updated choices; active/suspended turns and existing tabs retain theirs.
- The prototype must label all OS interaction as simulated and cover both themes, minimum-width layout, keyboard focus, validation states, install progress, cancellation, and retry.

## Acceptance criteria

The route is discoverable, both settings operate independently, error states remain reviewable, install completion follows the switch preference, and existing Settings navigation works. Build and diff checks pass. No production runtime changes occur during this stage.

## Open questions

Production supported shell families, release update policy, platform compatibility matrix, and exact SDK signatures need implementation-stage contract review.
