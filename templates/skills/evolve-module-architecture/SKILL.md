---
name: evolve-module-architecture
description: Use this skill when changing ownership boundaries, module responsibilities, subsystem splits, or command-surface design. Capture the intended architecture with `@module`/`@area`, attach implementation with `@implements`, and keep architecture separate from product specs and patterns.
---

# Evolve Module Architecture

## When To Use

Use this when a developer task changes how the codebase is organized:

- splitting or merging modules
- moving responsibilities between subsystems
- adding a new command or major internal boundary
- documenting who owns a file or component
- introducing Special architecture annotations to a repo

Do not use this to prove product behavior. Use specs for behavior and patterns for repeated implementation approaches.

## How To Use

1. Identify the boundary you are changing.
2. If Special is present, run `special arch MODULE.ID --verbose` or `special arch --metrics`.
3. If there is no module yet, add the smallest useful `@module ID` declaration.
4. Use `@area ID` only for structure.
5. Attach code with `@implements ID` or `@fileimplements ID`.
6. Edit the annotation where it is written when the module description is wrong.
7. Move or narrow implementation annotations when ownership is too broad or attached to the wrong code.
8. If the change also introduces a repeated approach, use `use-project-patterns`.

Example:

```rust
/**
@module EXPORT.CSV
Builds CSV export output from selected records and columns.
*/
// @fileimplements EXPORT.CSV
```

Useful commands:

```sh
special arch
special arch EXPORT.CSV --verbose
special arch EXPORT.CSV --metrics --verbose
special lint
```

## What To Do With Results

- If a current module has no implementation, add `@implements` or mark the module planned.
- If code implements the wrong module, move the annotation.
- If one file owns too much, split or narrow the module/implementation boundary.
- If a module description is really a product claim, move it to `@spec`.
- After editing, run `special arch` and `special lint` when available.
