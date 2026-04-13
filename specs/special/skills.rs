/**
@group SPECIAL.SKILLS.COMMAND
special skill discovery and installation commands.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.HELP
special skills prints explanatory help text that describes available skill ids and the supported `skills` command shapes without installing anything.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.HELP.NO_ROOT_WARNING
special skills does not emit implicit root-discovery warnings while printing overview help.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.HELP.INSTALL_DESTINATION_GUIDANCE
special skills describes the supported project, global, and custom install destination shapes without probing the current environment.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.EMITS_SKILL_TO_STDOUT
special skills SKILL_ID writes the selected skill contents to stdout without installing it.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND
special skills install is the command entrypoint for skill installation.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.ALL_SKILLS_DEFAULT
special skills install without a skill id installs all bundled skills.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.ONE_SKILL
special skills install SKILL_ID installs only the selected bundled skill.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.VALIDATES_SKILL_ID_BEFORE_PROMPT
special skills install reports an unknown skill id before prompting for destination or overwrite input.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.PROMPTS_FOR_DESTINATION
special skills install interactively prompts for the install destination with project, global, and custom options.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.REJECTS_UNKNOWN_PROMPT_CHOICES
special skills install rejects unknown interactive destination choices and reprompts instead of treating them as filesystem paths.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.NON_INTERACTIVE_DESTINATION
special skills install accepts `--destination` with `project`, `global`, or a custom path so installs can run without interactive destination prompts.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.FORCE_OVERWRITE
special skills install accepts `--force` with an explicit destination to overwrite conflicting installed skill directories without interactive confirmation.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.PROJECT_DESTINATION
when project install is selected, special skills install writes skills into `.agents/skills/` in the current repository root.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.GLOBAL_DESTINATION
when global install is selected, special skills install writes skills into `$CODEX_HOME/skills` and falls back to `~/.codex/skills` when `CODEX_HOME` is unset.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.CUSTOM_DESTINATION
when custom install is selected, special skills install prompts for a destination path and writes skills there.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.OVERWRITE_PROMPT
if the destination already contains a skill directory with the same name, special skills install prompts to overwrite that skill directory or abort the install.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.WRITES_PROJECT_SKILLS_DIRECTORY
special installs project-local skills into `.agents/skills/` in the current repository.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.USES_AGENT_SKILLS_LAYOUT
special installs skills as standard skill directories with `SKILL.md` and optional support files rather than inventing a special-only layout.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALLS_SHIP_CHANGE_SKILL
special installs a bundled `ship-product-change` skill with a primary `SKILL.md` whose frontmatter name is `ship-product-change`.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALLS_DEFINE_PRODUCT_SPECS_SKILL
special installs a bundled `define-product-specs` skill with a primary `SKILL.md` whose frontmatter name is `define-product-specs`.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALLS_VALIDATE_PRODUCT_CONTRACT_SKILL
special installs a bundled `validate-product-contract` skill with a primary `SKILL.md` whose frontmatter name is `validate-product-contract`.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALLS_VALIDATE_ARCHITECTURE_IMPLEMENTATION_SKILL
special installs a bundled `validate-architecture-implementation` skill with a primary `SKILL.md` whose frontmatter name is `validate-architecture-implementation`.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALLS_LIVE_STATE_SKILL
special installs a bundled `inspect-live-spec-state` skill with a primary `SKILL.md` whose frontmatter name is `inspect-live-spec-state`.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INSTALLS_PLANNED_WORK_SKILL
special installs a bundled `find-planned-work` skill with a primary `SKILL.md` whose frontmatter name is `find-planned-work`.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.BUNDLES_REFERENCES_FOR_PROGRESSIVE_DISCLOSURE
special bundles deeper skill guidance in sidecar references so startup skill summaries stay compact while richer instructions remain available on activation.
*/

/**
@spec SPECIAL.SKILLS.COMMAND.INCLUDES_TRIGGER_EVAL_FIXTURES
special includes trigger eval fixtures so skill descriptions can be checked against should-trigger and should-not-trigger examples.
*/
