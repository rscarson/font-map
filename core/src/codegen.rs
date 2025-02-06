//! Code generation utilities for fonts
use proc_macro2::TokenStream;
use quote::format_ident;
use std::{collections::HashMap, vec};

use crate::font::{Font, StringKind};

mod docstring;
use docstring::DocstringExt;

mod to_ident;
use to_ident::{to_categories, to_identifiers, ToIdentExt};

mod category;
use category::FontCategoryDesc;

mod glyph;
pub use glyph::GlyphDesc;

#[cfg(feature = "codegen")]
#[cfg_attr(docsrs, doc(cfg(feature = "codegen")))]
pub use quote::quote;

/// Describes a font used for code generation
#[derive(Debug, Clone)]
pub struct FontDesc {
    identifier: String,
    family: Option<String>,
    comments: Vec<String>,
    categories: Vec<FontCategoryDesc>,
}
impl FontDesc {
    /// Describe the font from a `Font` instance, optionally skipping categories
    pub fn from_font(identifier: &str, font: &Font, skip_categories: bool) -> Self {
        let identifier = identifier.to_string();
        let family = font.string(StringKind::FontFamily).map(ToString::to_string);
        let mut comments = font.gen_docblock();

        //
        // Get initial categories
        let mut categories = if skip_categories {
            // If set, skip categorization all-together
            let glyphs = to_identifiers(font.glyphs());
            vec![FontCategoryDesc::new(&identifier, glyphs)]
        } else {
            // Otherwise, attempt a best-effort categorization
            let raw_categories = to_categories(font.glyphs());
            let mut categories = Vec::with_capacity(raw_categories.len());
            for (name, glyphs) in raw_categories {
                categories.push(FontCategoryDesc::new(&name, glyphs));
            }

            categories
        };

        //
        // If we have just one, fall-back to single-cat generation
        if categories.len() == 1 {
            let category = &mut categories[0];
            category.set_name(identifier.clone());
            category.set_comments(comments.drain(..));

            return Self {
                identifier,
                family,
                comments,
                categories,
            };
        }

        //
        // Extract (or create) the `Other` category
        let mut other = categories
            .iter()
            .position(|c| c.name() == "Other")
            .map_or_else(
                || FontCategoryDesc::new("Other", HashMap::default()),
                |idx| categories.swap_remove(idx),
            );

        //
        // Extract all categories with < 3 glyphs and merge them with `Other`
        categories = categories
            .drain(..)
            .filter_map(|category| {
                if category.glyphs().len() > 2 {
                    return Some(category);
                }

                let (name, glyphs) = category.into_inner();
                for mut glyph in glyphs {
                    let identifier = name.merge_identifiers(glyph.identifier());
                    glyph.set_identifier(identifier);
                    other.insert(glyph);
                }
                None
            })
            .collect();

        //
        // Create an All category, populated with every glyph
        let mut all = FontCategoryDesc::new("All", HashMap::default());
        all.extend(other.glyphs().iter().cloned());
        for category in &categories {
            let glyphs = category.glyphs().iter();
            all.extend(glyphs.map(|glyph| {
                let mut glyph = glyph.clone();
                let identifier = category.name().merge_identifiers(glyph.identifier());
                glyph.set_identifier(identifier);
                glyph
            }));
        }

        //
        // Sort the modified glyph cats
        all.sort();
        other.sort();

        //
        // Sort the categories by name
        categories.sort_by(|a, b| a.name().cmp(b.name()));

        //
        // And update stuff
        other.update_comments();
        all.set_comments([format!(
            "Contains the full set of {} glyphs in the font.  ",
            all.glyphs().len()
        )]);

        //
        // Add All, Other to the start
        categories.insert(0, other);
        categories.insert(0, all);

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
    #[must_use]
    pub fn codegen(&self, extra_impl: Option<TokenStream>) -> TokenStream {
        let identifier = format_ident!("{}", &self.identifier);
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
            let mut categories = Vec::with_capacity(self.categories.len());
            for category in &self.categories {
                categories.push(category.codegen(None));
            }

            let mut variant_names = Vec::with_capacity(categories.len());
            let mut variants = Vec::with_capacity(categories.len());
            for category in &self.categories {
                let name = format_ident!("{}", category.name());
                let comments = category.comments();
                let variant = quote! {
                    #( #[doc = #comments] )*
                    #name(categories :: #name),
                };

                variant_names.push(name);
                variants.push(variant);
            }

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
