# Requirement

## Background

Session archive currently does not provide a complete active/archive workflow in the desktop client.

## Goals

- Require confirmation before archive, restore, and permanent deletion.
- Keep archived sessions in a bottom overlay drawer and out of the active list.
- Allow archived sessions to be inspected read-only and restored into an editable session.
- Permanently delete a session, its child sessions, durable records, and SunCode-managed image files.

## Non-goals

- Deleting user source files referenced by session images.
- Changing project archive behavior.

## Requirements

- Running or waiting sessions cannot be archived.
- Archived sessions cannot be renamed or submitted to.
- Restore returns the session to the active list and selects it as an editable session.
- Permanent deletion is physical deletion and includes delegated child records.

## Acceptance criteria

- Rust, C ABI, C# SDK, and Avalonia expose the active/archive/delete workflow.
- Focused tests cover state guards, cascade deletion, image cleanup, and desktop projection.

## Open questions

- None.
