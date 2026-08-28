# Universal Components

This layer owns cross-platform interaction primitives and their review states. Each component folder owns its React specimen and a stable `index.js` export. `UniversalComponentsPage.jsx` is the module index, while `modules/<module>/` owns an independently routable page that composes the relevant specimens. Module pages must not duplicate component implementation.

The Avalonia client remains the runtime owner of presentation. This directory must not become a second production component library.
