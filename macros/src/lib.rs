use font_map_core::{codegen::FontDesc, font::Font};
use proc_macro::TokenStream;
use syn::{parse::Parse, parse_macro_input, Ident, Lit, LitStr};

struct FontParameters {
    identifier: Ident,
    path: LitStr,
    skip_categories: bool,
}
impl Parse for FontParameters {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let identifier = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let path = input.parse()?;

        let mut skip_categories = false;

        while input.parse::<syn::Token![,]>().is_ok() {
            let name = input.parse::<Ident>()?;
            input.parse::<syn::Token![=]>()?;
            let value = input.parse::<Lit>()?;

            match name {
                n if n == "skip_categories" => match value {
                    Lit::Bool(b) => skip_categories = b.value,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            value,
                            "Expected a boolean value for `skip_categories`",
                        ))
                    }
                },

                _ => {
                    return Err(syn::Error::new_spanned(
                        name,
                        "Unknown parameter, expected `skip_categories`",
                    ))
                }
            }
        }

        Ok(Self {
            identifier,
            path,
            skip_categories,
        })
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

    let generator = FontDesc::from_font(&identifier, &font, input.skip_categories);
    generator.codegen(None).into()
}
