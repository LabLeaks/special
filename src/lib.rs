/**
@module SPECIAL
Repository root application surface for `special`. This module wires together the command boundary and the core spec, module, render, and discovery subsystems without owning their internal rules.
*/
// @fileimplements SPECIAL
mod annotation_syntax;
mod cli;
mod config;
mod discovery;
mod extractor;
mod index;
mod model;
mod modules;
mod parser;
mod planned_syntax;
mod render;
mod skills;

pub use cli::run_from_env;
