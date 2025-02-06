use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::font::{Font, StringKind};

mod docstring;
use docstring::DocstringExt;

mod to_ident;
use to_ident::{to_categories, to_identifiers};

mod category;
use category::FontCategoryDesc;

/// Describes a font used for code generation
#[derive(Debug, Clone)]
pub struct FontDesc {
    identifier: Ident,
    family: Option<String>,
    comments: Vec<String>,
    categories: Vec<FontCategoryDesc>,
}
impl FontDesc {
    /// Describe the font from a `Font` instance, optionally skipping categories
    pub fn from_font(name: &str, font: &Font, skip_categories: bool) -> Self {
        let identifier = Ident::new(name, Span::call_site());
        let family = font.string(StringKind::FontFamily).map(ToString::to_string);
        let mut comments = font.gen_docblock();

        let glyphs = font.glyphs();
        let mut categories = if skip_categories {
            let glyphs = to_identifiers(glyphs);
            vec![FontCategoryDesc::new(name, glyphs)]
        } else {
            to_categories(glyphs)
                .into_iter()
                .map(|(name, glyphs)| FontCategoryDesc::new(&name, glyphs))
                .collect()
        };

        if categories.len() == 1 {
            let category = &mut categories[0];
            category.set_name(name);
            category.set_comments(comments.drain(..));
        } else {
            //
            // Extract the 'Other' category if it exists
            let other = categories.iter().position(|c| c.name() == "Other");
            let other = other.map(|idx| categories.remove(idx));
            let mut other =
                other.unwrap_or_else(|| FontCategoryDesc::new("Other", HashMap::default()));

            //
            // Search for categories with < 3 glyphs and merge them with Uncategorized
            let mut uncategorized = vec![];
            let mut i = 0;
            while i < categories.len() {
                let category = &mut categories[i];
                if category.glyphs().len() < 3 {
                    let mut contents: Vec<_> = category.glyphs_mut().drain().collect();

                    // Each name should be `category_name` + `glyph_name`
                    for (name, _) in &mut contents {
                        *name = format!("{}{name}", category.name());
                    }

                    uncategorized.extend(contents);
                    categories.remove(i);
                } else {
                    i += 1;
                }
            }
            if !uncategorized.is_empty() {
                other.glyphs_mut().extend(uncategorized);
            }

            //
            // Create an All category
            let mut all = HashMap::with_capacity(glyphs.len());
            for category in &categories {
                let glyphs = category.glyphs().iter();
                all.extend(glyphs.map(|(n, g)| {
                    // If name starts with `_`, strip it
                    let category = category.name();
                    let name = n.strip_prefix('_').unwrap_or(n);

                    let name = format!("{category}{name}");
                    (name, g.clone())
                }));
            }
            let glyphs = other.glyphs().iter();
            all.extend(glyphs.map(|(n, g)| (n.clone(), g.clone())));

            //
            // Sort the categories by name
            categories.sort_by(|a, b| a.name().cmp(b.name()));

            //
            // And update stuff
            other.update_comments();
            let mut all = FontCategoryDesc::new("All", all);
            all.set_comments([format!(
                "Contains the full set of {} glyphs in the font.  ",
                all.glyphs().len()
            )]);

            //
            // Add All, Other to the start
            categories.insert(0, other);
            categories.insert(0, all);
        }

        Self {
            identifier,
            family,
            comments,
            categories,
        }
    }

    /// Returns true if this font has only one category
    #[must_use]
    pub fn is_single_category(&self) -> bool {
        self.categories.len() == 1
    }

    /// Generate the code for the font
    ///
    /// Optionally, you can inject additional code into the generated font's impl
    #[allow(clippy::needless_pass_by_value)]
    pub fn codegen(&self, extra_impl: Option<TokenStream>) -> TokenStream {
        let identifier = &self.identifier;
        let outer_comments = &self.comments;
        let font_family = self.family.iter();
        let injection = extra_impl.iter();

        if self.is_single_category() {
            let category = &self.categories[0];

            category.codegen(Some(quote! {
                #(
                    /// The family name for font
                    pub const FONT_FAMILY: &str = #font_family;
                )*
            }))
        } else {
            //
            // Categories in a module, generate an outer wrapper enum
            let categories: Vec<TokenStream> = self.categories.iter().map(Into::into).collect();

            let variant_names: Vec<_> =
                self.categories.iter().map(FontCategoryDesc::name).collect();

            let variants = self.categories.iter().map(|cat| {
                let name = cat.name();
                let comments = cat.comments();
                quote! {
                    #( #[doc = #comments] )*
                    #name(categories :: #name),
                }
            });

            quote! {
                /// Contains a set of enums for each of the sub-categories in this font
                pub mod categories {
                    #( #categories )*
                }

                #[allow(rustdoc::bare_urls)]
                #( #[doc = #outer_comments] )*
                #[doc = ""]
                #[doc = "See the [`categories`] module for more information."]
                #[derive(Debug, Clone, Copy)]
                #[rustfmt::skip]
                pub enum #identifier {
                    #( #variants )*
                }

                #[rustfmt::skip]
                #[allow(dead_code)]
                impl #identifier {
                    #(
                        /// The family name for this glyph's font
                        pub const FONT_FAMILY: &str = #font_family;
                    )*

                    /// Returns the postscript name of the glyph
                    #[allow(clippy::too_many_lines)]
                    #[allow(clippy::match_same_arms)]
                    #[must_use]
                    pub fn name(&self) -> &'static str {
                        match self {
                            #( Self :: #variant_names(inner) => inner.name(), )*
                        }
                    }

                    #(
                        #injection
                    )*
                }

                #(
                    impl From<categories :: #variant_names> for #identifier {
                        fn from(value: categories :: #variant_names) -> Self {
                            Self :: #variant_names(value)
                        }
                    }
                )*

                impl From<#identifier> for char {
                    fn from(value: #identifier) -> Self {
                        match value {
                            #( #identifier :: #variant_names(inner) => char::from(inner), )*
                        }
                    }
                }

                impl From<&#identifier> for char {
                    fn from(value: &#identifier) -> Self {
                        (*value).into()
                    }
                }

                impl From<#identifier> for u32 {
                    fn from(value: #identifier) -> Self {
                        match value {
                            #( #identifier :: #variant_names(inner) => inner as u32, )*
                        }
                    }
                }

                impl From<&#identifier> for u32 {
                    fn from(value: &#identifier) -> Self {
                        (*value).into()
                    }
                }

                impl std::fmt::Display for #identifier {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self {
                            #( #identifier :: #variant_names(inner) => inner.fmt(f), )*
                        }
                    }
                }
            }
        }
    }
}

impl From<&FontDesc> for TokenStream {
    fn from(value: &FontDesc) -> Self {
        value.codegen(None)
    }
}
