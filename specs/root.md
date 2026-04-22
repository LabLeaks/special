
### `@group SPECIAL`
special top-level structure.

### `@group SPECIAL.ROADMAP`
Future planned extensions for special.

### `@spec SPECIAL.ROADMAP.SIGNED_ATTESTATIONS`
### `@planned`
special accepts and surfaces signed attestation records.

### `@spec SPECIAL.ROADMAP.ATTESTATION_TTL_ENFORCEMENT`
### `@planned`
special enforces attestation review intervals in CI.

### `@spec SPECIAL.ROADMAP.CLAIM_HASH_BINDING`
### `@planned`
special binds attestations to spec revision hashes.

### `@spec SPECIAL.ROADMAP.SUPPORT_POLICY`
### `@planned`
special can require verifies or attests for configured spec subtrees.

### `@spec SPECIAL.ROADMAP.SPEC_DIFFS`
### `@planned`
special supports diff views for spec graph changes.

### `@spec SPECIAL.ROADMAP.PLANNED_RELEASE_WARNINGS`
### `@planned`
special warns when the current project version exactly matches a planned spec's release target string.

### `@group SPECIAL.ROADMAP.TRACEABILITY`
Future planned implementation traceability across owned code, verifying tests, and spec state.

### `@spec SPECIAL.ROADMAP.TRACEABILITY.FILE_VERIFY_DISCONNECTS`
### `@planned`
special surfaces suspiciously broad file-scoped verification artifacts when tests in the same file reach disconnected implementation clusters.

### `@spec SPECIAL.ROADMAP.TRACEABILITY.STABLE_ITEM_IDS`
### `@planned`
special assigns stable implementation item identifiers so traceability can distinguish duplicate item names without relying on ambiguous name-only matching.

### `@spec SPECIAL.ROADMAP.TRACEABILITY.PROCESS_BOUNDARY_INVOCATIONS`
### `@planned`
special traces implementation through explicit process-boundary invocation edges, such as local binary execution, when analyzers can resolve the invoked entry surface honestly.

### `@spec SPECIAL.ROADMAP.TRACEABILITY.PYTHON`
### `@planned 0.8.0`
special health surfaces built-in Python implementation traceability for analyzable Python source items when the shipped Python bridge and toolchain contract are honest enough to enable.

### `@spec SPECIAL.ROADMAP.TRACEABILITY.PYTHON.TOOL_EDGES`
### `@planned 0.8.0`
special health combines parser and Python local object-flow edges so imported constructors, local assignments, and `partial(...)`-produced instances can trace to the correct owned implementation items.

### `@spec SPECIAL.ROADMAP.TRACEABILITY.PYTHON.REFERENCE_EDGES`
### `@planned 0.8.0`
special health combines parser and Python tool-backed reference edges so callback-style Python support can trace to the owned implementation item that is passed through an intermediary helper.

### `@group SPECIAL.ROADMAP.MODULE_METRICS`
Future planned built-in module-metrics extensions.

### `@spec SPECIAL.ROADMAP.MODULE_METRICS.PYTHON`
### `@planned 0.8.0`
special arch --metrics surfaces built-in Python implementation evidence for owned Python code, including public and internal item counts plus per-item connected, isolated, and unreached signals when the shipped Python analyzer can identify them honestly.

### `@group SPECIAL.QUALITY`
special internal quality contracts.

### `@group SPECIAL.CONFIG`
special configuration and root discovery.

### `@group SPECIAL.INIT`
special project initialization workflow.

### `@group SPECIAL.SKILLS`
special project skill installation workflow.

### `@group SPECIAL.DISTRIBUTION`
special release and installation contract.
