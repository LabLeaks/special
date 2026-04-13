/**
@module SPECIAL
Repository root application surface for `special`.
*/
// @implements SPECIAL
mod annotation_syntax;
mod cli;
mod config;
mod extractor;
mod index;
mod model;
mod modules;
mod parser;
mod planned_syntax;
mod render;
mod skills;

pub use cli::run_from_env;
