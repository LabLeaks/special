/**
@module SPECIAL.MODULES.ANALYZE.RUST.SURFACE
Extracts Rust-specific public and internal item counts from owned Rust implementation for architecture evidence.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.RUST.SURFACE
use syn::{ImplItem, Item, Visibility};

#[derive(Debug, Default)]
pub(super) struct RustSurfaceSummary {
    pub public_items: usize,
    pub internal_items: usize,
}

impl RustSurfaceSummary {
    pub(super) fn observe(&mut self, text: &str) {
        if let Ok(file) = syn::parse_file(text) {
            self.add_counts(&rust_surface_counts(&file.items));
            return;
        }

        if let Ok(item) = syn::parse_str::<Item>(text) {
            self.add_counts(&rust_surface_counts(std::slice::from_ref(&item)));
        }
    }

    fn add_counts(&mut self, counts: &RustSurfaceSummary) {
        self.public_items += counts.public_items;
        self.internal_items += counts.internal_items;
    }
}

fn rust_surface_counts(items: &[Item]) -> RustSurfaceSummary {
    let mut counts = RustSurfaceSummary::default();
    for item in items {
        match item {
            Item::Const(item) => count_visibility(&item.vis, &mut counts),
            Item::Enum(item) => count_visibility(&item.vis, &mut counts),
            Item::ExternCrate(item) => count_visibility(&item.vis, &mut counts),
            Item::Fn(item) => count_visibility(&item.vis, &mut counts),
            Item::Macro(_) => counts.internal_items += 1,
            Item::Mod(item) => {
                count_visibility(&item.vis, &mut counts);
                if let Some((_, nested)) = &item.content {
                    counts.add_counts(&rust_surface_counts(nested));
                }
            }
            Item::Static(item) => count_visibility(&item.vis, &mut counts),
            Item::Struct(item) => count_visibility(&item.vis, &mut counts),
            Item::Trait(item) => count_visibility(&item.vis, &mut counts),
            Item::TraitAlias(item) => count_visibility(&item.vis, &mut counts),
            Item::Type(item) => count_visibility(&item.vis, &mut counts),
            Item::Union(item) => count_visibility(&item.vis, &mut counts),
            Item::Use(item) => count_visibility(&item.vis, &mut counts),
            Item::Impl(item) => {
                for impl_item in &item.items {
                    match impl_item {
                        ImplItem::Fn(method) => count_visibility(&method.vis, &mut counts),
                        ImplItem::Const(constant) => count_visibility(&constant.vis, &mut counts),
                        ImplItem::Type(ty) => count_visibility(&ty.vis, &mut counts),
                        ImplItem::Macro(_) | ImplItem::Verbatim(_) | _ => {}
                    }
                }
            }
            Item::ForeignMod(item) => {
                if item.abi.name.is_some() {
                    counts.internal_items += 1;
                }
            }
            Item::Verbatim(_) => {}
            _ => counts.internal_items += 1,
        }
    }
    counts
}

fn count_visibility(vis: &Visibility, counts: &mut RustSurfaceSummary) {
    match vis {
        Visibility::Public(_) => counts.public_items += 1,
        _ => counts.internal_items += 1,
    }
}
