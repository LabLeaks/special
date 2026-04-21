/**
@module SPECIAL.MODULES.ANALYZE.RUST.ITEM_METRICS
Computes shared per-item Rust evidence used across complexity, quality, and item-signal analysis so built-in Rust metrics stay consistent.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.RUST.ITEM_METRICS
use syn::visit::Visit;
use syn::{BinOp, FnArg, GenericArgument, ImplItemFn, Item, PathArguments, Type, Visibility};

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct RustItemMetrics {
    pub public: bool,
    pub root_visible: bool,
    pub parameter_count: usize,
    pub bool_parameter_count: usize,
    pub raw_string_parameter_count: usize,
    pub cyclomatic: usize,
    pub cognitive: usize,
    pub panic_site_count: usize,
}

pub(super) fn function_metrics(
    visibility: &Visibility,
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
    block: &syn::Block,
) -> RustItemMetrics {
    RustItemMetrics {
        public: is_public_visibility(visibility),
        root_visible: is_root_visibility(visibility),
        parameter_count: parameter_count(inputs),
        bool_parameter_count: bool_parameter_count(inputs),
        raw_string_parameter_count: raw_string_parameter_count(inputs),
        cyclomatic: function_cyclomatic(block),
        cognitive: function_cognitive(block),
        panic_site_count: panic_site_count(block),
    }
}

pub(super) fn method_metrics(method: &ImplItemFn) -> RustItemMetrics {
    function_metrics(&method.vis, &method.sig.inputs, &method.block)
}

pub(super) trait RustItemObserver {
    fn observe_item(&mut self, item: &Item);
}

pub(super) fn observe_rust_text(observer: &mut impl RustItemObserver, text: &str) {
    visit_rust_items(text, |item| observer.observe_item(item));
}

pub(super) fn visit_rust_items(text: &str, mut visit_item: impl FnMut(&Item)) {
    if let Ok(file) = syn::parse_file(text) {
        for item in &file.items {
            visit_item(item);
        }
        return;
    }

    if let Ok(item) = syn::parse_str::<Item>(text) {
        visit_item(&item);
    }
}

pub(super) fn parameter_count(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
) -> usize {
    inputs
        .iter()
        .filter(|arg| matches!(arg, FnArg::Receiver(_) | FnArg::Typed(_)))
        .count()
}

fn bool_parameter_count(inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>) -> usize {
    inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(argument) => Some(&*argument.ty),
            FnArg::Receiver(_) => None,
        })
        .filter(|ty| is_bool_type(ty))
        .count()
}

fn raw_string_parameter_count(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
) -> usize {
    inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(argument) => Some(&*argument.ty),
            FnArg::Receiver(_) => None,
        })
        .filter(|ty| is_raw_string_type(ty))
        .count()
}

fn is_public_visibility(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

fn is_root_visibility(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_) | Visibility::Restricted(_))
}

fn is_bool_type(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "bool"),
        Type::Reference(reference) => is_bool_type(&reference.elem),
        _ => false,
    }
}

fn is_raw_string_type(ty: &Type) -> bool {
    match ty {
        Type::Reference(reference) => is_raw_string_type(&reference.elem),
        Type::Path(type_path) => {
            let Some(segment) = type_path.path.segments.last() else {
                return false;
            };
            if segment.ident == "str" || segment.ident == "String" {
                return true;
            }
            if segment.ident == "Cow"
                && let PathArguments::AngleBracketed(arguments) = &segment.arguments
            {
                return arguments.args.iter().any(|argument| match argument {
                    GenericArgument::Type(inner) => is_raw_string_type(inner),
                    _ => false,
                });
            }
            false
        }
        _ => false,
    }
}

fn function_cyclomatic(block: &syn::Block) -> usize {
    let mut visitor = CyclomaticVisitor { complexity: 1 };
    visitor.visit_block(block);
    visitor.complexity
}

struct CyclomaticVisitor {
    complexity: usize,
}

impl CyclomaticVisitor {
    fn visit_loop_like(&mut self, visit_children: impl FnOnce(&mut Self)) {
        self.complexity += 1;
        visit_children(self);
    }
}

impl Visit<'_> for CyclomaticVisitor {
    fn visit_expr_if(&mut self, node: &syn::ExprIf) {
        self.complexity += 1;
        syn::visit::visit_expr_if(self, node);
    }

    fn visit_expr_match(&mut self, node: &syn::ExprMatch) {
        self.complexity += node.arms.len().max(1);
        syn::visit::visit_expr_match(self, node);
    }

    fn visit_expr_for_loop(&mut self, node: &syn::ExprForLoop) {
        self.visit_loop_like(|visitor| syn::visit::visit_expr_for_loop(visitor, node));
    }

    fn visit_expr_while(&mut self, node: &syn::ExprWhile) {
        self.visit_loop_like(|visitor| syn::visit::visit_expr_while(visitor, node));
    }

    fn visit_expr_loop(&mut self, node: &syn::ExprLoop) {
        self.visit_loop_like(|visitor| syn::visit::visit_expr_loop(visitor, node));
    }

    fn visit_expr_binary(&mut self, node: &syn::ExprBinary) {
        if matches!(node.op, BinOp::And(_) | BinOp::Or(_)) {
            self.complexity += 1;
        }
        syn::visit::visit_expr_binary(self, node);
    }
}

fn function_cognitive(block: &syn::Block) -> usize {
    let mut visitor = CognitiveVisitor::default();
    visitor.visit_block(block);
    visitor.complexity
}

#[derive(Default)]
struct CognitiveVisitor {
    complexity: usize,
    nesting: usize,
}

impl CognitiveVisitor {
    fn bump_structural(&mut self) {
        self.complexity += 1 + self.nesting;
    }

    fn with_nested<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.nesting += 1;
        f(self);
        self.nesting = self.nesting.saturating_sub(1);
    }

    fn visit_nested_loop_body(
        &mut self,
        visit_head: impl FnOnce(&mut Self),
        visit_body: impl FnOnce(&mut Self),
    ) {
        self.bump_structural();
        visit_head(self);
        self.with_nested(visit_body);
    }
}

impl Visit<'_> for CognitiveVisitor {
    fn visit_expr_if(&mut self, node: &syn::ExprIf) {
        self.bump_structural();
        self.visit_expr(&node.cond);
        self.with_nested(|visitor| visitor.visit_block(&node.then_branch));
        if let Some((_, else_branch)) = &node.else_branch {
            self.with_nested(|visitor| visitor.visit_expr(else_branch));
        }
    }

    fn visit_expr_match(&mut self, node: &syn::ExprMatch) {
        self.bump_structural();
        self.visit_expr(&node.expr);
        self.with_nested(|visitor| {
            for arm in &node.arms {
                visitor.visit_arm(arm);
            }
        });
    }

    fn visit_expr_for_loop(&mut self, node: &syn::ExprForLoop) {
        self.visit_nested_loop_body(
            |visitor| visitor.visit_expr(&node.expr),
            |visitor| visitor.visit_block(&node.body),
        );
    }

    fn visit_expr_while(&mut self, node: &syn::ExprWhile) {
        self.visit_nested_loop_body(
            |visitor| visitor.visit_expr(&node.cond),
            |visitor| visitor.visit_block(&node.body),
        );
    }

    fn visit_expr_loop(&mut self, node: &syn::ExprLoop) {
        self.bump_structural();
        self.with_nested(|visitor| visitor.visit_block(&node.body));
    }

    fn visit_expr_binary(&mut self, node: &syn::ExprBinary) {
        if matches!(node.op, BinOp::And(_) | BinOp::Or(_)) {
            self.complexity += 1;
        }
        syn::visit::visit_expr_binary(self, node);
    }
}

fn panic_site_count(block: &syn::Block) -> usize {
    let mut visitor = PanicVisitor { count: 0 };
    visitor.visit_block(block);
    visitor.count
}

struct PanicVisitor {
    count: usize,
}

impl Visit<'_> for PanicVisitor {
    fn visit_macro(&mut self, node: &syn::Macro) {
        if node.path.is_ident("panic")
            || node.path.is_ident("todo")
            || node.path.is_ident("unimplemented")
            || node.path.is_ident("unreachable")
        {
            self.count += 1;
        }
        syn::visit::visit_macro(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &syn::ExprMethodCall) {
        if node.method == "unwrap" || node.method == "expect" {
            self.count += 1;
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}
