use std::fs;
use std::path::Path;

use anyhow::Result;

struct SkillFile {
    path: &'static str,
    contents: &'static str,
}

const SKILL_FILES: &[SkillFile] = &[
    SkillFile {
        path: ".agents/skills/ship-features-with-product-specs/SKILL.md",
        contents: r#"---
name: ship-features-with-product-specs
description: Use this skill when adding a feature or changing behavior in a project where product specs should stay honest. Add or update the relevant `special` specs first, then keep each live claim matched to one honest verify.
---

# Ship Features With Product Specs

Use this skill when feature work or behavior changes need to stay aligned with product specs, whether the repo already uses `special` or you are introducing it now.

1. Start from the product change, not the code. If the repo already uses `special`, find the relevant claim with `special spec` or `special spec --all`. If it does not, start by defining the claim you are about to ship.
2. If the change is not ready to ship, add or keep `@planned` on the exact claim instead of over-claiming.
3. If the claim is live, make sure it has one honest, self-contained `@verifies` or `@attests` artifact.
4. Run `special spec SPEC.ID --verbose` before trusting a verify. Read the attached body and decide whether it actually proves the claim.
5. Treat `@group` as structure only. Parent `@spec` claims still need their own direct support or `@planned`.
6. If the feature changes command behavior, prefer command-boundary verifies over helper-only tests.

Read [references/feature-workflow.md](references/feature-workflow.md) for the detailed workflow and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
"#,
    },
    SkillFile {
        path: ".agents/skills/ship-features-with-product-specs/references/feature-workflow.md",
        contents: r#"# Feature Workflow

Use this checklist when shipping a feature in a repo that uses `special`:

1. Find the exact product-spec claim that the feature changes.
2. If the claim does not exist yet, add it before or alongside the implementation.
3. Keep the claim present tense and narrow enough that one verify can honestly support it.
4. Use `special spec SPEC.ID --verbose` to inspect the existing support before changing code.
5. Tighten weak verifies until a reviewer can judge the claim locally.
6. If the work is not ready to ship, keep the claim planned instead of pretending it is live.
7. Re-run `special spec` or `special lint` after the change to confirm the tree still materializes cleanly.
"#,
    },
    SkillFile {
        path: ".agents/skills/ship-features-with-product-specs/references/trigger-evals.md",
        contents: r#"# Trigger Evals

## Should Trigger

 - Add a feature to this repo and update the product specs as part of the change.
 - We are starting to use product specs in this project; add the first claims before you implement the feature.
- This behavior changed; make sure the relevant `special` claims and verifies stay honest.
- I need to ship this command change without drifting from the product spec.
- Add the missing planned/live spec for this new behavior before you implement it.

## Should Not Trigger

- Fix this Rust borrow-checker error.
- Summarize the latest release notes.
- Add syntax highlighting to the HTML output.
- Set up GitHub Actions caching for cargo builds.
"#,
    },
    SkillFile {
        path: ".agents/skills/write-product-specs/SKILL.md",
        contents: r#"---
name: write-product-specs
description: Use this skill when creating or revising product specs for a project. Write present-tense claims, use `@group` only for structure, and keep each live spec narrow enough for one self-contained verify.
---

# Write Product Specs

Use this skill when you are creating, editing, or tightening product specs, whether you are working in an existing `special` repo or introducing the workflow now.

1. Write claim text in present tense. Shipping a planned claim should not require rewriting the sentence.
2. Use `@group` for structure-only nodes and `@spec` for real claims.
3. Keep `@planned` local to the exact claim that is not live yet.
4. Split claims until each live `@spec` can point to one honest, self-contained `@verifies` or `@attests` artifact.
5. Prefer product-boundary verifies for product-boundary behavior. Do not let helper tests carry a command-level claim.
6. If a parent claim says something real, give it direct support. Child support does not justify a parent `@spec`.

Read [references/spec-writing.md](references/spec-writing.md) for the writing rubric and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
"#,
    },
    SkillFile {
        path: ".agents/skills/write-product-specs/references/spec-writing.md",
        contents: r#"# Spec Writing Rubric

Use this rubric when writing or tightening product specs in `special`:

- State the contract, not the implementation.
- Keep the claim narrow enough that one verify can honestly support it.
- Avoid future tense. `@planned` already carries the future state.
- Avoid umbrella claims that only read like folders; use `@group` for those.
- Keep user-facing behavior at the command boundary and verify it there.
- Use exact wording that can stay stable after the claim ships.
- If a claim is not ready, keep it planned rather than overfitting a weak verify.
"#,
    },
    SkillFile {
        path: ".agents/skills/write-product-specs/references/trigger-evals.md",
        contents: r#"# Trigger Evals

## Should Trigger

- Write the product spec for this new feature before we implement it.
- Rewrite this claim so it states the contract instead of the implementation.
- Split this parent `@spec` into a structural `@group` and direct child claims.
- Tighten these specs so each live claim has one honest, self-contained verify.

## Should Not Trigger

- Investigate a production outage in the API service.
- Convert this markdown file to HTML.
- Install the published binary with Homebrew.
- Review whether a verify body is self-contained enough.
"#,
    },
];

pub fn install_project_skills(root: &Path) -> Result<usize> {
    for file in SKILL_FILES {
        let path = root.join(file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, file.contents)?;
    }

    Ok(SKILL_FILES
        .iter()
        .filter(|file| file.path.ends_with("/SKILL.md"))
        .count())
}
