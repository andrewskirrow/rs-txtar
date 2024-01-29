//! The txtar crate implements a trivial text-based file archive format.
//! This has been ported from the [go package of the same name](https://pkg.go.dev/golang.org/x/tools/txtar).
//!
//! The goals for the format are:
//!
//!   - be trivial enough to create and edit by hand.
//!   - be able to store trees of text files describing go command test cases.
//!   - diff nicely in git history and code reviews.
//!
//! Non-goals include being a completely general archive format,
//! storing binary data, storing file modes, storing special files like
//! symbolic links, and so on.
//!
//! # Txtar format
//!
//! A txtar archive is zero or more comment lines and then a sequence of file entries.
//! Each file entry begins with a file marker line of the form "-- FILENAME --"
//! and is followed by zero or more file content lines making up the file data.
//! The comment or file content ends at the next file marker line.
//! The file marker line must begin with the three-byte sequence "-- "
//! and end with the three-byte sequence " --", but the enclosed
//! file name can be surrounding by additional white space,
//! all of which is stripped.
//!
//! If the txtar file is missing a trailing newline on the final line,
//! parsers should consider a final newline to be present anyway.
//!
//! There are no possible syntax errors in a txtar archive.
//!
//! ```text
//! Comment1
//! Comment 2 is here
//! -- file 1 --
//! This is the
//! content of file 1
//! -- file2  --
//! This is the conten of file 2
//! ```
//! # Examples
//!
//! You can use `Archive::from` to convert a string to an archive
//! ```
//! use rs_txtar::Archive;
//!
//! let tx_str = "comment1
//! comment2
//! -- file1 --
//! This is file 1
//! -- file2 --
//! this is file2
//! ";
//!
//! let archive = Archive::from(tx_str);
//!
//! assert_eq!(archive.comment, "comment1\ncomment2\n");
//!
//! assert!(archive.contains("file1"));
//! assert_eq!(archive["file2"].name.as_str(), "file2");
//! assert_eq!(archive["file2"].content.as_str(), "this is file2\n");
//! assert!(archive.get("file2").is_some());
//!
//! assert!(!archive.contains("not-exists"));
//! assert!(archive.get("not-exists").is_none());
//!```

#[non_exhaustive]
/// A txtar archive
pub struct Archive {
    /// The comments from the archive
    pub comment: String,

    /// The files in the archive
    pub files: Vec<File>,
}

impl Archive {
    /// Create a new archive
    pub fn new() -> Self {
        Archive {
            comment: String::new(),
            files: Vec::new(),
        }
    }

    /// Read an archive from the file specified by `path`
    pub fn from_file(path: &str) -> Result<Self, std::io::Error> {
        let mut f = std::fs::File::open(path)?;
        Archive::read(&mut f)
    }

    /// Read an archive from the specified `reader`
    pub fn read(reader: &mut impl std::io::Read) -> Result<Self, std::io::Error> {
        let mut s = String::new();
        reader.read_to_string(&mut s)?;
        Ok(Archive::from(s.as_str()))
    }

    /// Return `true` if `self` contains the file `name`
    pub fn contains(&self, name: &str) -> bool {
        self.files.iter().any(|f| f.name.as_str() == name)
    }

    /// Return `Some(file)` if the archive contains a file named `name`. Otherwise retuens `None`.
    /// You can also use `archive[name]` if you want a panic when the file doesn't exist
    pub fn get(&self, name: &str) -> Option<&File> {
        self.files.iter().find(|f| f.name.as_str() == name)
    }
}

impl Default for Archive {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a [&str] into an archive
impl From<&str> for Archive {
    /// parses `value` into a archive.
    fn from(value: &str) -> Self {
        let (comment, mut file_name, mut after) = find_next_marker(value);
        let mut files = Vec::new();
        while !file_name.is_empty() {
            let (data, next_file, after_next) = find_next_marker(after);

            let content = fix_newline(data);

            let file = File::new(file_name, &content);
            files.push(file);

            file_name = next_file;
            after = after_next;
        }

        Archive {
            comment: comment.to_owned(),
            files,
        }
    }
}

impl std::ops::Index<&str> for Archive {
    type Output = File;

    /// Return the file named `index` or panics if there is no sych file. You can also use
    /// [Archive::get] to avoid panicking the file isn't in the archive.
    fn index(&self, index: &str) -> &Self::Output {
        match self.files.iter().find(|f| f.name.as_str() == index) {
            Some(f) => f,
            None => panic!("Archive doesn't contain file: {}", index),
        }
    }
}

/// A file that resides in a txtar [Archive]
#[non_exhaustive]
pub struct File {
    /// The name of the file
    pub name: String,

    /// The file content
    pub content: String,
}

impl File {
    /// Create a new file
    pub fn new(name: &str, content: &str) -> File {
        File {
            name: name.to_owned(),
            content: content.to_owned(),
        }
    }
}

const MARKER: &str = "-- ";
const NEWLINE_MARKER: &str = "\n-- ";
const MARKER_END: &str = " --";

/// Finds the next file marker in `s`
/// If there is a file marker returns (beforeMarker, fileName, afterMarker)
/// Otherwise returns (s, "", "")
fn find_next_marker(s: &str) -> (&str, &str, &str) {
    let mut i = 0;
    loop {
        let (name, after) = parse_leading_marker(&s[i..]);
        if !name.is_empty() {
            return (&s[0..i], name, after);
        }

        if let Some(index) = s[i..].find(NEWLINE_MARKER) {
            i += index + 1;
        } else {
            return (s, "", "");
        }
    }
}

fn fix_newline(s: &str) -> String {
    let mut owned = s.to_owned();
    if !owned.is_empty() && !owned.ends_with('\n') {
        owned.push('\n');
    }
    owned
}

/// if `s` starts with a file marker then returns `(fileName, dataAfterLine)`
/// Otherwisw returns ("", "")
fn parse_leading_marker(s: &str) -> (&str, &str) {
    if !s.starts_with(MARKER) {
        return ("", "");
    }

    let (mut data, after) = match s.split_once('\n') {
        None => (s, ""),
        Some((x, y)) => (x, y),
    };

    if data.ends_with('\r') {
        data = &data[0..data.len() - 1];
    }

    if !data.ends_with(MARKER_END) || data.len() <= (MARKER.len() + MARKER_END.len()) {
        return ("", "");
    }

    let name = data[MARKER.len()..data.len() - MARKER_END.len()].trim();
    (name, after)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn do_parse_test(s: &'static str, expect: Archive) {
        let parsed = Archive::from(s);
        assert_eq!(
            parsed.comment, expect.comment,
            "parsed.comment == expected.comment"
        );

        let compare_max = usize::min(parsed.files.len(), expect.files.len());
        for i in 0..compare_max {
            let parsed_file = &parsed.files[i];
            let expected_file = &expect.files[i];

            assert_eq!(
                parsed_file.name, expected_file.name,
                "parsed.files[{}].name == expected.files[{}].name",
                i, i
            );

            assert_eq!(
                parsed_file.content, expected_file.content,
                "parsed.files[{}].content == expected.files[{}].content",
                i, i
            );
        }
    }

    macro_rules! parse_test {
        ($name:ident, $str:expr, $expect:expr) => {
            #[test]
            fn $name() {
                do_parse_test($str, $expect);
            }
        };
    }

    parse_test!(
        parse_basic,
        r"comment1
comment2
-- file1 --
File 1 text.
-- foo ---
More file 1 text.
-- file 2 --
File 2 text.
-- empty --
-- empty filename line --
some content
-- --
-- noNL --
hello world",
        Archive {
            comment: "comment1\ncomment2\n".to_owned(),
            files: vec![
                File::new("file1", "File 1 text.\n-- foo ---\nMore file 1 text.\n"),
                File::new("file 2", "File 2 text.\n"),
                File::new("empty", ""),
                File::new("empty filename line", "some content\n-- --\n"),
                File::new("noNL", "hello world\n"),
            ]
        }
    );
}
