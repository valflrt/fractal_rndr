pub struct Path {
    s: String,
}

impl Path {
    pub fn new(s: &str) -> Path {
        Path { s: s.to_string() }
    }

    pub fn extension(&self) -> Option<&str> {
        self.s
            .rfind('.')
            .take_if(|pos| self.s[*pos..].contains(['/', '\\']))
            .map(|pos| &self.s[pos + 1..])
    }

    // pub fn replace_extension(&self, new_extension: &str) -> Path {
    //     Path {
    //         s: match self.s.rfind('.') {
    //             Some(pos) => {
    //                 if self.s[pos..].contains('/') || self.s[pos..].contains('\\') {
    //                     format!("{}.{}", self.s, new_extension)
    //                 } else {
    //                     format!("{}{}", &self.s[..pos], new_extension)
    //                 }
    //             }
    //             None => {
    //                 format!("{}.{}", self.s, new_extension)
    //             }
    //         },
    //     }
    // }
}

impl AsRef<std::path::Path> for Path {
    fn as_ref(&self) -> &std::path::Path {
        self.s.as_ref()
    }
}
