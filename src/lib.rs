/**
@module SPECIAL
Repository root application surface for `special`. This module wires together the bare `special` overview plus the `special specs`, `special arch`, and `special health` command surfaces, along with the underlying spec, architecture, render, and discovery subsystems, without owning their internal rules.
*/
// @fileimplements SPECIAL
mod annotation_syntax;
mod cache;
mod cli;
mod config;
mod discovery;
mod extractor;
mod id_path;
mod index;
mod language_packs;
mod model;
mod modules;
mod overview;
mod parser;
mod planned_syntax;
mod render;
mod skills;
mod syntax;

pub use cli::run_from_env;
