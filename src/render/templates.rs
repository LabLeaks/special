/**
@module SPECIAL.RENDER.TEMPLATES
Provides the shared template-rendering helpers used by text and HTML backends so render layout lives in template files rather than handwritten string assembly.
*/
// @fileimplements SPECIAL.RENDER.TEMPLATES
use askama::Template;

pub(super) fn render_template<T: Template>(template: &T) -> String {
    template
        .render()
        .expect("render templates should not fail for in-memory data")
}

pub(super) fn text_indent(depth: usize) -> String {
    "  ".repeat(depth)
}
