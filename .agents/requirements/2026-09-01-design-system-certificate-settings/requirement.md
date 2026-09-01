# Requirement

## Background

The design-system settings page already shows the global HTTPS verification toggle, but it does not yet represent certificate-source controls beneath that toggle. Reviewers need to evaluate how system certificates and custom certificate files are selected before production implementation work begins.

## Goals

- Extend the design-system Network settings specimen with certificate-source controls.
- Show how a custom certificate path is enabled only when the user opts out of system certificates.
- Keep the change limited to design documentation and the React review browser.

## Non-goals

- Change the Avalonia desktop client.
- Change Rust HTTPS behavior, storage, or SDK contracts.
- Implement actual certificate loading or validation.

## Requirements

- The Network settings panel must keep the existing HTTPS verification toggle.
- When certificate verification is enabled, the panel must show a subordinate certificate configuration group.
- That group must include a toggle for using system certificates.
- That group must include a certificate path selector styled like `SCFileSelector`.
- When system certificates are enabled, the certificate path selector must be visibly disabled.
- When system certificates are disabled, the certificate path selector must be enabled and require a custom path.
- The certificate path selector must use a file-picker style interaction, not a folder picker.
- The design-system build must succeed and `git diff --check` must succeed.

## Edge cases

- Long certificate paths must remain readable without horizontal overflow.
- The disabled certificate path must still communicate why it cannot be edited.
- Custom mode must remain understandable even before a path is chosen.

## Acceptance criteria

- The Network settings specimen clearly shows system-certificate and custom-certificate modes.
- The custom certificate path control visually matches the existing path-selector language.
- The certificate path control disables correctly when system certificates are enabled.
- The design-system build succeeds.
- `git diff --check` succeeds.

## Open questions

- None.
