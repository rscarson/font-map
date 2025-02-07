//! # font-map
//! ## Font parser / enumerator with support for code generation
//!
//! [![Crates.io](https://img.shields.io/crates/v/font-map.svg)](https://crates.io/crates/font-map/)
//! [![Build Status](https://github.com/rscarson/font-map/actions/workflows/tests.yml/badge.svg?branch=master)](https://github.com/rscarson/font-map/actions?query=branch%3Amaster)
//! [![docs.rs](https://img.shields.io/docsrs/font-map)](https://docs.rs/font-map/latest/)
//! [![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/rscarson/font-map/master/LICENSE)
//!
//! This crate provides functionality for parsing font files and enumerating the glyphs they contain.
//!
//! The base usecase for this crate is to create an enum of all the glyphs in a font file,  
//! for use in fontend projects, where you want to refer to glyphs by name rather than by codepoint:
//!
//! ```rust
//! use font_map::font;
//!
//! font!(Icon, "google_material_symbols/font.ttf");
//!
//! const DELETE: Icon = Icon::Delete;
//! ```
//!
//! The generated code includes information for each glyph, such as:
//! - codepoint, and postfix-name
//! - Plus a generated SVG preview image visible on hover
//!
//! You can also access `Icon::FONT_FAMILY` to simplify font usage in your frontend.
//!
//! -----
//!
//! Another use is to use it for introspection of font files:
//!
//! ```rust
//! use font_map::font::Font;
//!
//! # use font_map::error::ParseError;
//! # fn main() -> Result<(), ParseError> {
//! let font = Font::from_file("google_material_symbols/font.ttf")?;
//! if let Some(glyph) = font.glyph_named("delete") {
//!     let codepoint = glyph.codepoint();
//!     let svg = glyph.svg_preview();
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//! - `macros` - Enables the `font!` macro for code generation
//! - `codegen` - Enables the `FontCodegenExt` trait for runtime code generation
//! - `extended-svg` - Enables compressed and base64 encoded SVG data in the generated code (Needed for image previews)
//!
//! ## Known Limitations
//! This crate was made for a very specific use-case, and as such currently has a few limitations:
//! - Only supports TTF fonts
//! - And even then, only a subset of the spec, namely:
//! - Only some formats of the `cmap` table
//! - Only Unicode, or MS encoding 1 and 10, and `Macintosh::0` of the `name` table
//! - Only formats 2.5 or below of the `post` table
//!
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![cfg_attr(docsrs, feature(doc_cfg))]
pub use font_map_core::*;

#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
pub use font_map_macros::*;

/// **Only designed to be used inside `build.rs`**
///
/// This macro is used to generate the code for a font file, and set up the build script to rerun
/// if the font file changes.
///
/// The generated code will include an enum with all the glyphs in the font, optionally split by
/// category
///
/// To include the generated code, see `[font_map::include_font]`
///
/// # Example
/// ```no_run
/// use font_map::build_font;
///
/// fn main() {
///     build_font!(
///         path = "../examples/slick.ttf",
///         name = SlickFont,
///         skip_categories = false, /* Can be omitted - if `true`, generate one giant enum instead of a set of categories */
///     );
/// }
/// ```
#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[allow(clippy::needless_doctest_main)]
#[macro_export]
macro_rules! build_font {
    (
        path = $path:literal,
        name = $name:ident,
        skip_categories = $skip_categories:literal $(,)?
    ) => {
        const FONT_BYTES: &[u8] = include_bytes!($path);
        println!(concat!("cargo:rerun-if-changed=", $path));

        let target_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let target_path = std::path::Path::new(&target_dir)
            .join($path)
            .display()
            .to_string();

        //
        // Load the font and perform code generation
        let font = font_map::font::Font::new(FONT_BYTES).expect("Bundled font was invalid!");
        let generator =
            font_map::codegen::FontDesc::from_font(stringify!($name), &font, $skip_categories);
        let code = generator
            .codegen(Some(font_map::codegen::quote! {
                /// The raw bytes of the font file
                pub const FONT_BYTES: &[u8] = include_bytes!(#target_path);
            }))
            .to_string();

        //
        // Create the target file
        let dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
        let target =
            std::path::Path::new(&dir).join(&format!("font_generated_{}.rs", stringify!($name)));
        std::fs::write(&target, code).expect("Failed to write generated icon-enum");

        //
        // Provide an ENV var with the path to the generated file
        println!(
            concat!("cargo:rustc-env=FONT_GEN_", stringify!($name), "={}"),
            target.display()
        );
    };

    (
        path = $path:literal,
        name = $name:ident $(,)?
    ) => {
        $crate::build_font! {
            path = $path,
            name = $name,
            skip_categories = false
        }
    };
}

/// Includes a font file generated by the [`build_font!`] macro
///
/// **NOTE:** Due to existing issues with rust-analyzer you may need to restart the RA server (left side of bottom toolbar)
/// after adding a new font file
///
/// This macro will include the generated code for the font's symbols, and provide:
/// - `FONT_BYTES`: The raw bytes of the font file
/// - `load_font()`: A function that returns a `font_map::font::Font` instance describing the font and its symbols
///
/// # Example
/// ```ignore
/// use font_map::include_font;
///
/// include_font!(GoogleMaterialSymbols);
///
/// const DELETE: GoogleMaterialSymbols = GoogleMaterialSymbols::Delete;
/// ```
#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[macro_export]
macro_rules! include_font {
    ($name:ident) => {
        //
        // Generated font bindings
        include!(env!(concat!("FONT_GEN_", stringify!($name))));

        /// Returning a `font_map::Font` instance describing the font and its symbols
        #[allow(
            clippy::missing_panics_doc,
            reason = "The panic message is clear enough"
        )]
        #[must_use]
        pub fn load_font() -> font_map::font::Font {
            font_map::font::Font::new($name::FONT_BYTES).expect("Bundled font was invalid!")
        }
    };
}
