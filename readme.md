
# Txtar

The txtar crate implements a trivial text-based file archive format.
This has been ported from the go package of the same name.
                                                                                   
The goals for the format are:

  - be trivial enough to create and edit by hand.
  - be able to store trees of text files describing go command test cases.
  - diff nicely in git history and code reviews.
                                                                                   
Non-goals include being a completely general archive format,
storing binary data, storing file modes, storing special files like
symbolic links, and so on.
                                                                                   
# Txtar format
                                                                                   
A txtar archive is zero or more comment lines and then a sequence of file entries.
Each file entry begins with a file marker line of the form "-- FILENAME --"
and is followed by zero or more file content lines making up the file data.
The comment or file content ends at the next file marker line.
The file marker line must begin with the three-byte sequence "-- "
and end with the three-byte sequence " --", but the enclosed
file name can be surrounding by additional white space,
all of which is stripped.
                                                                                   
If the txtar file is missing a trailing newline on the final line,
parsers should consider a final newline to be present anyway.
                                                                                   
There are no possible syntax errors in a txtar archive.
                                                                                   
```text
Comment1
Comment 2 is here
-- file 1 --
This is the
content of file 1
-- file2  --
This is the conten of file 2
```

# Reading Examples

You can use `Archive::from` to convert a string to an archive

```rust
use txtar::Archive;
                                                           
let tx_str = "comment1
comment2
-- file1 --
This is file 1
-- file2 --
this is file2
";
                                                           
let archive = Archive::from(tx_str);
assert_eq!(archive.comment, "comment1\ncomment2\n");
``
