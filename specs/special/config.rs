/**
@spec SPECIAL.CONFIG.SPECIAL_TOML
special uses `special.toml` as an explicit project anchor when present.
*/

/**
@spec SPECIAL.CONFIG.SPECIAL_TOML.EXPLICIT_ROOT
special uses the optional `root` value in `special.toml` as the explicit project root, resolved relative to the config file.
*/

/**
@spec SPECIAL.CONFIG.SPECIAL_TOML.ANCESTOR_CONFIG
when launched from a nested subdirectory, special finds the nearest ancestor `special.toml` and resolves `root = "."` relative to that config file rather than the launch directory.
*/

/**
@spec SPECIAL.CONFIG.SPECIAL_TOML.OPTIONAL
special does not require `special.toml` for basic operation.
*/

/**
@spec SPECIAL.CONFIG.SPECIAL_TOML.SUPPRESSES_IMPLICIT_ROOT_WARNING
when `special.toml` determines the project root, special does not emit implicit-root discovery warnings.
*/

/**
@spec SPECIAL.CONFIG.SPECIAL_TOML.KEY_VALUE_SYNTAX
special exits with an error when `special.toml` contains a non-empty, non-comment line that does not use `key = "value"` syntax.
*/

/**
@spec SPECIAL.CONFIG.SPECIAL_TOML.QUOTED_STRING_VALUES
special exits with an error when `special.toml` uses an unquoted value.
*/

/**
@spec SPECIAL.CONFIG.SPECIAL_TOML.UNKNOWN_KEYS
special exits with an error when `special.toml` uses an unknown key.
*/

/**
@spec SPECIAL.CONFIG.SPECIAL_TOML.DUPLICATE_KEYS_REJECTED
special exits with an error when `special.toml` declares the same key more than once.
*/

/**
@spec SPECIAL.CONFIG.SPECIAL_TOML.EXISTING_ROOT_REQUIRED
special exits with an error when `special.toml` points `root` at a path that does not exist.
*/

/**
@spec SPECIAL.CONFIG.SPECIAL_TOML.ROOT_MUST_BE_DIRECTORY
special exits with an error when `special.toml` points `root` at a file instead of a directory.
*/

/**
@spec SPECIAL.CONFIG.SPECIAL_TOML.ROOT_MUST_NOT_BE_EMPTY
special exits with an error when `special.toml` sets `root` to an empty string.
*/

/**
@spec SPECIAL.CONFIG.SPECIAL_TOML.VERSION
special reads an optional `version` key from `special.toml` to select parser and linter rules.
*/

/**
@spec SPECIAL.CONFIG.SPECIAL_TOML.VERSION.DEFAULTS_TO_LEGACY
without a `version` key in `special.toml`, special defaults `@planned` to backward-looking within-block ownership instead of version 1's adjacent ownership.
*/

/**
@spec SPECIAL.CONFIG.SPECIAL_TOML.VERSION.UNKNOWN_REJECTED
special exits with an error when `special.toml` uses an unsupported `version` value.
*/

/**
@spec SPECIAL.CONFIG.SPECIAL_TOML.VERSION.MISSING_WARNS_AND_ASSUMES_LEGACY
when `special.toml` omits `version`, special emits a lint warning that it is using compatibility parsing rules and points to `version = "1"` for the current rules.
*/

/**
@group SPECIAL.CONFIG.ROOT_DISCOVERY
special discovers a project root from explicit config, VCS state, or the current directory.
*/

/**
@spec SPECIAL.CONFIG.ROOT_DISCOVERY.VCS_DEFAULT
without `special.toml`, special prefers the nearest enclosing VCS root.
*/

/**
@spec SPECIAL.CONFIG.ROOT_DISCOVERY.CWD_FALLBACK
without `special.toml` or VCS metadata, special falls back to the current working directory.
*/

/**
@spec SPECIAL.CONFIG.ROOT_DISCOVERY.IMPLICIT_ROOT_WARNING
without `special.toml`, special warns that it is using an implicit root for discovery.
*/

/**
@spec SPECIAL.CONFIG.ROOT_DISCOVERY.NO_CONFIG_VERSION_WARNING
without `special.toml`, special emits a lint warning that it is using compatibility parsing rules and points to `special init` for creating current config.
*/
