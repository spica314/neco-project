# Design Principles

## Dependency Policy

For regular dependencies (not dev-dependencies used only for testing), we maintain a policy of using only the standard library and avoiding external crates. This keeps the compiler implementation minimal and self-contained.

Test dependencies and development tools may use external crates as needed for effective testing and development workflows.
