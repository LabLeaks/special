# Trigger Evals

## Should Trigger

- Check whether `APP.RENDER` is actually implemented by the code attached to it.
- Review whether this module is honestly implemented or if the `@implements` tags are hand-wavy.
- Use `special modules APP.CLI.SKILLS --verbose` and tell me if the implementation matches the architecture intent.
- Use `special modules APP.PARSER --metrics --verbose` and tell me what looks suspicious inside the claimed boundary.
- Run the metrics against this module and surface any unusual or outward-heavy items without inventing violations.
- Find live modules that still have no direct implementation attached.

## Should Not Trigger

- Check whether this spec is really supported by its verify.
- Rewrite this product claim so it states shipped behavior instead of implementation.
- Find planned work for the next release.
- Install the CLI from Homebrew.
