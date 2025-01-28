pub trait ToIdentExt {
    /// Converts a font string to a valid Rust identifier
    /// Font strings use . - _ and alphanumeric characters
    fn to_identifier(&self) -> String;
}
impl ToIdentExt for str {
    fn to_identifier(&self) -> String {
        //
        // Replace all occurrences of . and - with _
        let mut identifier = self.replace(['.', '-'], "_");

        //
        // Replace all _[a-z] pairs with the uppercase letter
        let mut chars = identifier.chars();
        let mut new_identifier = String::with_capacity(identifier.len());
        while let Some(c) = chars.next() {
            if c == '_' {
                if let Some(next) = chars.next() {
                    new_identifier.push(next.to_ascii_uppercase());
                } else {
                    new_identifier.push(c);
                }
            } else {
                new_identifier.push(c);
            }
        }
        identifier = new_identifier;

        //
        // If the identifier starts with a digit, prepend an underscore
        match identifier.chars().next() {
            // If the string is empty, `_` is a valid identifier
            None => return "_".to_string(),

            Some(c) if c.is_ascii_digit() => {
                // Prepend an underscore for digits
                identifier.insert(0, '_');
            }

            Some(c) => {
                // Set first character to uppercase
                let first = c.to_string().to_uppercase();
                identifier = first + &identifier[1..];
            }
        }

        //
        // If the identifier is a reserved keyword, prepend an underscore
        if identifier.as_str() == "Self" {
            identifier.insert(0, '_');
        }

        identifier
    }
}
