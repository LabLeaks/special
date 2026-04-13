/**
@spec SPECIAL.INIT.CREATES_SPECIAL_TOML
special init creates `special.toml` in the current directory with `version = "1"` and `root = "."`.
*/

/**
@spec SPECIAL.INIT.DOES_NOT_OVERWRITE_SPECIAL_TOML
special init fails instead of overwriting an existing `special.toml` in the current directory.
*/

/**
@spec SPECIAL.INIT.REJECTS_NESTED_ACTIVE_CONFIG
when the current directory is already governed by an ancestor `special.toml`, special init fails instead of creating a nested config by accident.
*/
