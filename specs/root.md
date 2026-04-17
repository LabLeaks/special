
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

### `@spec SPECIAL.ROADMAP.FILE_ATTESTS`
### `@planned`
special supports explicit file-scoped attestation artifacts with `@fileattests`.

### `@spec SPECIAL.ROADMAP.DEPRECATION_LIFECYCLE`
### `@planned`
special supports lightweight `@deprecated <release>` metadata so live claims can be marked for retirement before their support artifacts and final claim text are removed.

### `@group SPECIAL.ROADMAP.TRACEABILITY`
Future planned implementation traceability across owned code, verifying tests, and spec state.

### `@spec SPECIAL.ROADMAP.TRACEABILITY.IMPL_TO_VERIFIED_SPEC`
### `@planned`
special traces owned implementation through verifying tests to the spec claims those tests verify.

### `@spec SPECIAL.ROADMAP.TRACEABILITY.SPEC_STATE_CLASSIFICATION`
### `@planned`
special distinguishes owned implementation traced only to planned or deprecated spec claims from implementation traced to live claims.

### `@spec SPECIAL.ROADMAP.TRACEABILITY.UNKNOWN_IMPLEMENTATION`
### `@planned`
special surfaces owned implementation with no path through a verifying test to any spec claim.

### `@spec SPECIAL.ROADMAP.TRACEABILITY.FILE_VERIFY_DISCONNECTS`
### `@planned`
special surfaces suspiciously broad file-scoped verification artifacts when tests in the same file reach disconnected implementation clusters.

### `@spec SPECIAL.ROADMAP.TRACEABILITY.STABLE_ITEM_IDS`
### `@planned`
special assigns stable implementation item identifiers so traceability can distinguish duplicate item names without relying on ambiguous name-only matching.

### `@spec SPECIAL.ROADMAP.TRACEABILITY.PROCESS_BOUNDARY_INVOCATIONS`
### `@planned`
special traces implementation through explicit process-boundary invocation edges, such as local binary execution, when analyzers can resolve the invoked entry surface honestly.

### `@group SPECIAL.ROADMAP.MODULE_ANALYSIS`
Future planned expansion of built-in language analysis providers.

### `@spec SPECIAL.ROADMAP.MODULE_ANALYSIS.TYPESCRIPT`
### `@planned`
special repo --experimental surfaces built-in TypeScript implementation traceability for owned TypeScript code.

### `@spec SPECIAL.ROADMAP.MODULE_ANALYSIS.GO`
### `@planned`
special repo --experimental surfaces built-in Go implementation traceability for owned Go code.

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
