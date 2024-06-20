use alloc::format;
use alloc::string::{String, ToString};

pub(crate) fn pluralize<Num, ToPluralize, OnEmpty>(n: Num, word_to_pluralize: ToPluralize, on_empty: OnEmpty) -> String
    where Num: Into<usize>, ToPluralize: AsRef<str>, OnEmpty: AsRef<str>
{
    let res = match n.into() {
        0 => return on_empty.as_ref().to_string(),
        1 => format!("1 {}", word_to_pluralize.as_ref()),
        n => format!("{n} {}s", word_to_pluralize.as_ref())
    };
    return res;
}

pub(crate) fn join_strings<Strings: Iterator<Item=Item>, Item: AsRef<str>>(separator: &str, strings: Strings) -> String {
    let mut res = String::new();
    let mut is_first_string = true;
    strings.for_each(|string| {
        if !is_first_string {
            res.push_str(separator)
        }
        res.push_str(string.as_ref());
        is_first_string = false;
    });
    res
}

pub(crate) fn ident_lines_except_first(prefixed_contents: String, mut spaces: usize) -> String {
    let mut spacing = String::new();
    while spaces > 0 {
        spacing.push_str(" ");
        spaces -= 1;
    }
    let mut is_first_line = true;

    let spaced_contents = join_strings("\n", prefixed_contents.lines().map(|line| {
        let res = if is_first_line { line.to_string() } else { spacing.clone() + line };
        is_first_line = false;
        res
    }));
    spaced_contents
}
