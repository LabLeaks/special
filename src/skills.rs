/**
@module SPECIAL.SKILLS
Bundled skill catalog and install-facing asset definitions in `src/skills.rs`. This module owns which bundled skills ship with the binary and the files each skill installation materializes.
*/
// @fileimplements SPECIAL.SKILLS
mod install;

use anyhow::Result;

pub(crate) use install::{
    conflicting_skill_paths, install_bundled_skills, resolve_global_skills_root,
};

pub struct SkillAsset {
    relative_path: &'static str,
    contents: &'static str,
}

pub struct BundledSkill {
    pub id: &'static str,
    pub summary: &'static str,
    assets: &'static [SkillAsset],
}

const SHIP_PRODUCT_CHANGE_ASSETS: &[SkillAsset] = &[
    SkillAsset {
        relative_path: "SKILL.md",
        contents: include_str!("../templates/skills/ship-product-change/SKILL.md"),
    },
    SkillAsset {
        relative_path: "references/change-workflow.md",
        contents: include_str!(
            "../templates/skills/ship-product-change/references/change-workflow.md"
        ),
    },
    SkillAsset {
        relative_path: "references/trigger-evals.md",
        contents: include_str!(
            "../templates/skills/ship-product-change/references/trigger-evals.md"
        ),
    },
];

const DEFINE_PRODUCT_SPECS_ASSETS: &[SkillAsset] = &[
    SkillAsset {
        relative_path: "SKILL.md",
        contents: include_str!("../templates/skills/define-product-specs/SKILL.md"),
    },
    SkillAsset {
        relative_path: "references/spec-writing.md",
        contents: include_str!(
            "../templates/skills/define-product-specs/references/spec-writing.md"
        ),
    },
    SkillAsset {
        relative_path: "references/trigger-evals.md",
        contents: include_str!(
            "../templates/skills/define-product-specs/references/trigger-evals.md"
        ),
    },
];

const VALIDATE_PRODUCT_CONTRACT_ASSETS: &[SkillAsset] = &[
    SkillAsset {
        relative_path: "SKILL.md",
        contents: include_str!("../templates/skills/validate-product-contract/SKILL.md"),
    },
    SkillAsset {
        relative_path: "references/validation-checklist.md",
        contents: include_str!(
            "../templates/skills/validate-product-contract/references/validation-checklist.md"
        ),
    },
    SkillAsset {
        relative_path: "references/trigger-evals.md",
        contents: include_str!(
            "../templates/skills/validate-product-contract/references/trigger-evals.md"
        ),
    },
];

const VALIDATE_ARCHITECTURE_IMPLEMENTATION_ASSETS: &[SkillAsset] = &[
    SkillAsset {
        relative_path: "SKILL.md",
        contents: include_str!("../templates/skills/validate-architecture-implementation/SKILL.md"),
    },
    SkillAsset {
        relative_path: "references/validation-checklist.md",
        contents: include_str!(
            "../templates/skills/validate-architecture-implementation/references/validation-checklist.md"
        ),
    },
    SkillAsset {
        relative_path: "references/trigger-evals.md",
        contents: include_str!(
            "../templates/skills/validate-architecture-implementation/references/trigger-evals.md"
        ),
    },
];

const INSPECT_LIVE_SPEC_STATE_ASSETS: &[SkillAsset] = &[
    SkillAsset {
        relative_path: "SKILL.md",
        contents: include_str!("../templates/skills/inspect-live-spec-state/SKILL.md"),
    },
    SkillAsset {
        relative_path: "references/state-walkthrough.md",
        contents: include_str!(
            "../templates/skills/inspect-live-spec-state/references/state-walkthrough.md"
        ),
    },
    SkillAsset {
        relative_path: "references/trigger-evals.md",
        contents: include_str!(
            "../templates/skills/inspect-live-spec-state/references/trigger-evals.md"
        ),
    },
];

const FIND_PLANNED_WORK_ASSETS: &[SkillAsset] = &[
    SkillAsset {
        relative_path: "SKILL.md",
        contents: include_str!("../templates/skills/find-planned-work/SKILL.md"),
    },
    SkillAsset {
        relative_path: "references/planned-workflow.md",
        contents: include_str!(
            "../templates/skills/find-planned-work/references/planned-workflow.md"
        ),
    },
    SkillAsset {
        relative_path: "references/trigger-evals.md",
        contents: include_str!("../templates/skills/find-planned-work/references/trigger-evals.md"),
    },
];

const BUNDLED_SKILLS: &[BundledSkill] = &[
    BundledSkill {
        id: "ship-product-change",
        summary: "Ship a product change without drifting the contract.",
        assets: SHIP_PRODUCT_CHANGE_ASSETS,
    },
    BundledSkill {
        id: "define-product-specs",
        summary: "Turn requirements into explicit product specs.",
        assets: DEFINE_PRODUCT_SPECS_ASSETS,
    },
    BundledSkill {
        id: "validate-product-contract",
        summary: "Check whether one claim is honestly supported.",
        assets: VALIDATE_PRODUCT_CONTRACT_ASSETS,
    },
    BundledSkill {
        id: "validate-architecture-implementation",
        summary: "Check whether one module is honestly implemented.",
        assets: VALIDATE_ARCHITECTURE_IMPLEMENTATION_ASSETS,
    },
    BundledSkill {
        id: "inspect-live-spec-state",
        summary: "Inspect the current live validated spec state.",
        assets: INSPECT_LIVE_SPEC_STATE_ASSETS,
    },
    BundledSkill {
        id: "find-planned-work",
        summary: "Find planned product-spec work that is not live yet.",
        assets: FIND_PLANNED_WORK_ASSETS,
    },
];

pub fn bundled_skills() -> &'static [BundledSkill] {
    BUNDLED_SKILLS
}

pub fn bundled_skill(skill_id: &str) -> Option<&'static BundledSkill> {
    BUNDLED_SKILLS.iter().find(|skill| skill.id == skill_id)
}

pub fn primary_skill_contents(skill_id: &str) -> Result<&'static str> {
    let skill =
        bundled_skill(skill_id).ok_or_else(|| anyhow::anyhow!("unknown skill id `{skill_id}`"))?;
    skill
        .assets
        .iter()
        .find(|asset| asset.relative_path == "SKILL.md")
        .map(|asset| asset.contents)
        .ok_or_else(|| anyhow::anyhow!("bundled skill `{skill_id}` is missing `SKILL.md`"))
}
