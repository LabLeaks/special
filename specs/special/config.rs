/**
@spec SPECIAL.CONFIG.SPECIAL_TOML
special uses `special.toml` as an explicit project anchor when present.
*/

/**
@spec SPECIAL.CONFIG.SPECIAL_TOML.OPTIONAL
special does not require `special.toml` for basic operation.
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
