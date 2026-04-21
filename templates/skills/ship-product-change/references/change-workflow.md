# Change Workflow

Use this checklist when shipping a change that should keep the product contract honest.

This workflow is anchored in a few standard principles:

- Define and preserve the externally meaningful contract.
- Test behavior through public surfaces and observable results.
- Prefer tests that keep their value after internal refactors.

1. Identify the exact behavior that changed.
2. If the claim does not exist yet, add it before or alongside the implementation.
3. Keep the claim present tense and narrow enough that one verify can honestly support it.
4. Use `special specs SPEC.ID --verbose` to inspect the existing support before changing code.
5. Tighten weak verifies until a reviewer can judge the claim locally.
6. If the work is not ready to ship, keep the claim planned instead of pretending it is current.
7. If the work changes architecture intent or command-surface responsibilities, update the relevant source-local `@module` text in tracked source before or alongside implementation.
8. If the workflow includes bundled skill installation in this repo, prefer `special skills install ...` rather than manual copying. Treat `.agents/skills/` as runtime/install output, not tracked source.
9. Re-run `special specs`, `special arch`, `special health PATH...`, or `special lint` after the change to confirm the projections still materialize cleanly.
10. Use `special health PATH... --metrics` when the change may have cross-cutting code-health impact beyond the exact claim or module you edited.
11. Keep the contract focused on stable, externally meaningful invariants and avoid verifies that overfit transient details.

Good examples:

- Spec: `CSV exports include a header row with the selected column names.`
- Verify: exercise the export path and assert the first CSV row is the selected header row.
- Spec: `special skills install writes project-local installs under .agents/skills/.`
- Verify: run the install command and assert the destination directory and `SKILL.md` exist there.

Bad examples:

- Spec: `The help copy says "Select install destination".`
- Verify: assert an exact instructional paragraph in a bundled skill file.
- Spec: `The command calls helper parse_destination before install_bundled_skills.`

If the true contract is a side effect, interaction assertions can be part of the proof. Otherwise, prefer end-state and output checks over call-order checks.

Source anchors:

- W3C QA Specification Guidelines: clearer, more implementable, better testable specifications.
- Software Engineering at Google: test via public behavior and keep tests resilient to refactors.
- Testing Library guiding principle: prefer tests that resemble real use.

Example verify shape:

```python
# @verifies EXPORT.CSV.HEADERS
def test_csv_export_includes_selected_column_headers():
    csv_text = export_orders_csv(columns=["order_id", "status"])
    assert csv_text.splitlines()[0] == "order_id,status"
```
