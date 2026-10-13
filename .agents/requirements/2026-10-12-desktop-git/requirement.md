# Requirement

Desktop Source Control uses an installed, executable system Git. Hide all Git entry points and footer data when none is available; retain not-repository and retryable repository-error states when Git is available. Preserve read-only status, three diff scopes, rename/conflict/binary handling, project-relative scope, bounded output, cancellation, and patch copy. Remove Git operations, DTOs, bindings, and git2 production/test dependencies from Rust and SDKs. Keep agent bash policy, touched files, checkpoints, and undo. No Git mutations or installation.
