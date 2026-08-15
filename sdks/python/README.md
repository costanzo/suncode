# Python SDK

Planned package for Python consumers of the SunCode runtime.

This directory intentionally has no package implementation yet. When implemented, it will use PyO3 or the stable C ABI to embed the Rust SDK in the Python host process. It must not connect to an HTTP runtime service, open SQLite independently, contact model providers directly, or reimplement runtime behavior.
