# Bundled font licenses

The desktop build downloads these fonts into the ignored
`apps/desktop-avalonia/Assets/fonts/` directory and verifies the SHA-256
values in [`fonts.lock`](fonts.lock).
The font binaries are intentionally not stored in Git.

- **Noto Sans SC** — SIL Open Font License 1.1; source: [Google Fonts](https://github.com/google/fonts/tree/9710da1eacb3be272583c3224dcb70f9da6eadbb/ofl/notosanssc)
- **JetBrains Mono** — SIL Open Font License 1.1; source: [JetBrainsMono](https://github.com/JetBrains/JetBrainsMono/tree/19371302b95d218af43299bce79ddbddd0bc364d)
- **Seti UI** — MIT; source: [seti-ui](https://github.com/jesseweed/seti-ui/tree/2d6c5e68b4ded73c92dac291845ee44e1182d511)

The corresponding license texts are included in the upstream repositories and
are downloaded only as needed for release packaging. The existing Seti notice
is retained in `apps/desktop-avalonia/Assets/fonts/SETI-NOTICE.md`.
