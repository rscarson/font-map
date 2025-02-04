use font_map_core::{font::Font, FontEnum};
use proc_macro::TokenStream;
use syn::{parse::Parse, parse_macro_input, Ident, LitStr};

struct FontParameters {
    identifier: Ident,
    path: LitStr,
}
impl Parse for FontParameters {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let identifier = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let path = input.parse()?;

        Ok(Self { identifier, path })
    }
}

#[proc_macro]
pub fn font(input: TokenStream) -> TokenStream {
    //
    // Parse input as an ident, then a string literal - like:
    // font!(Icon, "path/to/font.ttf");
    let input = parse_macro_input!(input as FontParameters);

    let identifier = input.identifier.to_string();
    let path = input.path.value();

    let font_bytes =
        std::fs::read(&path).unwrap_or_else(|_| panic!("Failed to read font at `{path}`"));
    let font = Font::new(&font_bytes).unwrap_or_else(|_| panic!("Invalid font file: `{path}`"));

    FontEnum::from_font(&identifier, &font).codegen().into()
}
