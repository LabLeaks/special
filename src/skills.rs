use std::fs;
use std::path::Path;

use anyhow::Result;

struct SkillFile {
    path: &'static str,
    contents: &'static str,
}

const SKILL_FILES: &[SkillFile] = &[
    SkillFile {
        path: ".agents/skills/ship-product-change/SKILL.md",
        contents: include_str!("../templates/skills/ship-product-change/SKILL.md"),
    },
    SkillFile {
        path: ".agents/skills/ship-product-change/references/change-workflow.md",
        contents: include_str!("../templates/skills/ship-product-change/references/change-workflow.md"),
    },
    SkillFile {
        path: ".agents/skills/ship-product-change/references/trigger-evals.md",
        contents: include_str!("../templates/skills/ship-product-change/references/trigger-evals.md"),
    },
    SkillFile {
        path: ".agents/skills/define-product-specs/SKILL.md",
        contents: include_str!("../templates/skills/define-product-specs/SKILL.md"),
    },
    SkillFile {
        path: ".agents/skills/define-product-specs/references/spec-writing.md",
        contents: include_str!("../templates/skills/define-product-specs/references/spec-writing.md"),
    },
    SkillFile {
        path: ".agents/skills/define-product-specs/references/trigger-evals.md",
        contents: include_str!("../templates/skills/define-product-specs/references/trigger-evals.md"),
    },
    SkillFile {
        path: ".agents/skills/validate-product-contract/SKILL.md",
        contents: include_str!("../templates/skills/validate-product-contract/SKILL.md"),
    },
    SkillFile {
        path: ".agents/skills/validate-product-contract/references/validation-checklist.md",
        contents: include_str!("../templates/skills/validate-product-contract/references/validation-checklist.md"),
    },
    SkillFile {
        path: ".agents/skills/validate-product-contract/references/trigger-evals.md",
        contents: include_str!("../templates/skills/validate-product-contract/references/trigger-evals.md"),
    },
    SkillFile {
        path: ".agents/skills/inspect-live-spec-state/SKILL.md",
        contents: include_str!("../templates/skills/inspect-live-spec-state/SKILL.md"),
    },
    SkillFile {
        path: ".agents/skills/inspect-live-spec-state/references/state-walkthrough.md",
        contents: include_str!("../templates/skills/inspect-live-spec-state/references/state-walkthrough.md"),
    },
    SkillFile {
        path: ".agents/skills/inspect-live-spec-state/references/trigger-evals.md",
        contents: include_str!("../templates/skills/inspect-live-spec-state/references/trigger-evals.md"),
    },
    SkillFile {
        path: ".agents/skills/find-planned-work/SKILL.md",
        contents: include_str!("../templates/skills/find-planned-work/SKILL.md"),
    },
    SkillFile {
        path: ".agents/skills/find-planned-work/references/planned-workflow.md",
        contents: include_str!("../templates/skills/find-planned-work/references/planned-workflow.md"),
    },
    SkillFile {
        path: ".agents/skills/find-planned-work/references/trigger-evals.md",
        contents: include_str!("../templates/skills/find-planned-work/references/trigger-evals.md"),
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
