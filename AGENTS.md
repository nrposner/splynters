# Agent Guidelines for splynters

This document outlines the conventions and commands for agents working within the `splynters` repository.

## Build, Lint, and Test Commands

*   **Build**: `cargo build`
*   **Run all tests**: `cargo test`
*   **Run a single test**: `cargo test <test_name>` (e.g., `cargo test my_specific_test`)
*   **Run benchmarks**: `cargo bench`
*   **Lint**: `cargo clippy -- -D warnings` (treats warnings as errors)

## Code Style Guidelines (Rust)

*   **Imports**: Group related `use` statements.
*   **Formatting**: Adhere to `rustfmt` (run with `cargo fmt`).
*   **Naming Conventions**:
    *   Types (structs, enums): `PascalCase`
    *   Functions, methods, variables: `snake_case`
*   **Error Handling**: Prefer `Result<T, E>` for fallible operations. Provide meaningful error messages.
*   **Comments**: Use comments to explain *why* complex logic is implemented, not *what* it does.
