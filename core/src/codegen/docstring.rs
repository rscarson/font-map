use crate::font::{Font, StringKind};

pub trait DocstringExt {
    fn gen_docblock(&self) -> Vec<String>;
}

impl DocstringExt for Font {
    fn gen_docblock(&self) -> Vec<String> {
        let name = self.string(StringKind::FullFontName);
        let copyright = self.string(StringKind::CopyrightNotice);
        let description = self.string(StringKind::Description);

        let mut comments = Vec::new();

        if let Some(name) = name {
            comments.push(format!("{name}  "));
        }

        if let Some(copy) = copyright {
            comments.push(format!("{copy}  "));
        }

        if let Some(desc) = description {
            comments.push(format!("{desc}  "));
        }

        if !comments.is_empty() {
            comments.push(String::new());
        }

        comments.push(format!(
            "Contains the complete set of {} named glyphs for this font  ",
            self.glyphs().len()
        ));
        comments.push("Glyphs can be converted to their respective codepoints using `u32::from(*)`, or to `char` and `String` using `.into()`  ".to_string());
        comments
            .push("The postscript name for each glyph can be accessed using `.name()`".to_string());

        comments
    }
}
