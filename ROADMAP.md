# Orbis Engine Roadmap

This roadmap describes incremental architectural work on the `dev` branch. Each implementation step should be delivered as a small pull request with a narrow scope; avoid broad rewrites and new crates unless a demonstrated need justifies them.

## Principles

- Preserve existing behavior unless a PR explicitly changes it.
- Keep `orbis-engine` modular internally before considering additional crates.
- Keep Winit integration isolated in `winit_bridge`.
- Keep application/ECS state separate from GPU and UI implementation details.
- Prefer explicit ownership and data flow over public APIs that allow unsynchronized resource mutation.
- Add focused regression tests for each behavior changed.
- Validate with `cargo check --workspace` and `cargo test --workspace`; run platform CI for changes touching windowing or GPU setup.
- Do not add `cargo fmt` as a CI step; format changed Rust code before opening/updating a PR.

## Work plan

### 1. Establish correctness tests and fix existing diagnostics

**Goal:** create a reliable baseline before moving responsibilities.

- Strengthen tests around the existing render graph, including dependency ordering and cycle diagnostics.
- Fix the current cycle message, which reports the same pass as both reader and writer.
- Identify and test asset/resource lifecycle invariants: create, update, remove, and stale IDs.
- Record existing build/test results in each implementation PR; do not claim runtime behavior was tested unless it was run.

**Exit criteria:** regression tests assert useful failure messages and the workspace test suite passes in CI.

### 2. Clarify the frame lifecycle

**Goal:** make `Engine::tick` an understandable orchestration layer without changing visible behavior.

- Document the order of event handling, application update, resource synchronization, UI update, and rendering.
- Extract small private methods only where they establish a meaningful boundary.
- Keep Winit APIs inside `winit_bridge`.
- Preserve minimized-window behavior and add tests where feasible.

**Exit criteria:** the frame order is explicit, responsibilities are not duplicated, and CI passes.

### 3. Make CPU-to-GPU asset synchronization explicit

**Goal:** ensure GPU resources cannot silently diverge from the asset manager and UI registry.

- Define ownership and allowed mutation paths for GPU textures and other cached resources.
- Route create/update/remove operations through a single synchronization boundary.
- Handle removal and stale references deterministically.
- Add tests for registry and lifecycle invariants before introducing asynchronous loading.

**Exit criteria:** each resource lifecycle transition has one documented path and regression coverage.

### 4. Harden and integrate the render graph

**Goal:** evolve the current dependency sorter into a reliable renderer component.

- Validate read-before-write cases and ambiguous multiple writers.
- Make dependency ordering deterministic and improve cycle diagnostics.
- Test resource dependencies and invalid graphs independently of GPU execution.
- Integrate execution with the real render-pass context only after graph semantics are well-defined.

**Exit criteria:** graph validation is deterministic, testable, and reflects the actual pass/resource contract.

### 5. Improve ECS-derived data and frame cost

**Goal:** avoid repeated work without introducing premature complexity.

- Document which systems update transforms, hierarchy, visibility, and bounding boxes.
- Add tests for derived-data correctness.
- Profile before adding dirty tracking or incremental updates.
- Introduce incremental recomputation only where measurements justify it.

**Exit criteria:** correctness is covered and performance changes have a measurable rationale.

### 6. Strengthen the editor/backend boundary

**Goal:** keep UI code from owning application or renderer state.

- Keep UI communication based on commands, queries, and snapshots/DTOs.
- Keep scene/ECS state owned by the application layer.
- Aggregate statistics through DTOs rather than exposing renderer internals to widgets.
- Use the boundary to support future undo/redo and automation without coupling them to ImGui.

**Exit criteria:** editor operations can be tested through the command/query boundary without constructing UI widgets.

### 7. Add higher-level engine capabilities

Only after the foundations above are stable, evaluate:
- scene serialization and versioning;
- undo/redo;
- asynchronous asset loading and staged GPU uploads;
- GPU/resource profiling and debug views;
- transient render-resource allocation and lifetime tracking.

These are follow-up directions, not prerequisites for the initial refactoring.

## Pull request discipline

- One architectural concern per PR.
- Keep public API changes to a minimum.
- Do not combine formatting-only changes with behavior changes.
- State what was tested and what still needs manual verification.
- Do not merge dependent PRs until their base and CI status are correct.
