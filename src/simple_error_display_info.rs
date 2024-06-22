use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};

use crate::formatting::{ident_lines_except_first, join_strings, pluralize};

/// Holds information relative to an error in order to display it, and if the `serde` feature is
/// enabled, it also implements [serde::Serialize] and [serde::Deserialize], this is mostly to allow
/// you to save errors descriptions and send them in a standarized exchange data language like JSON
/// or Yaml.
///
/// The information this struct holds is:
/// * at (Optional): Where the errors happen, this is usually an input on a Parsing error, like an
/// AST-Building error.
/// * reason (Optional): What / Why the error happen.
/// * solution (Optional): How to solve the error.
/// * on_line_and_column (Optional): From which line and column the error happens.
/// * up_to_line_an_column (Optional): Upto which line and column the error happens.
/// * unexplained_causes (Default: 0): Number of causes from which their [SimpleErrorDisplayInfo]
/// contents were empty according to not matching [SimpleErrorDisplayInfo::is_explained].
/// * explained_causes (Vec of [SimpleErrorDisplayInfo]) : Causes that were actually explained.
///
/// When using the 'serde' feature, it also allows to both serialize and deserializing, being able
/// to hold information for errors on standard formats and probably using them for auditing later.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SimpleErrorDisplayInfo {
    /// Where the errors happen, this is usually an input on a Parsing error, like an AST-Building
    /// error.
    at: Option<String>,
    /// What / Why the error happen.
    reason: Option<String>,
    /// How to solve the error.
    solution: Option<String>,
    /// From which line and column the error happens.
    on_line_and_column: Option<(usize, usize)>,
    /// Upto which line and column the error happens.
    up_to_line_an_column: Option<(usize, usize)>,
    /// Number of causes from which their [SimpleErrorDisplayInfo] contents were empty according to
    /// not matching [SimpleErrorDisplayInfo::is_explained].
    unexplained_causes: usize,
    /// Displays of causes that were actually explained.
    explained_causes: Vec<SimpleErrorDisplayInfo>,
}

/// Implements display by calling [SimpleErrorDisplayInfo::as_display_string]
impl Display for SimpleErrorDisplayInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.as_display_string())
    }
}

impl SimpleErrorDisplayInfo {
    /// Creates a new value of [SimpleErrorDisplayInfo] giving:
    /// * at: Where the errors happen, this is usually an input on a Parsing error, like an
    ///       AST-Building error.
    /// * reason: What / Why the error happen.
    /// * solution: How to solve the error.
    /// * on_line_and_column: From which line and column the error happens.
    /// * up_to_line_an_column: Upto which line and column the error happens.
    /// * unexplained_causes: Number of causes from which their [SimpleErrorDisplayInfo] contents
    ///                       were empty according to not matching
    ///                       [SimpleErrorDisplayInfo::is_explained].
    /// * explained_causes: Displays of causes that were actually explained.
    pub(crate) const fn new(at: Option<String>, reason: Option<String>, solution: Option<String>, on_line_and_column: Option<(usize, usize)>, up_to_line_an_column: Option<(usize, usize)>, unexplained_causes: usize, explained_causes: Vec<SimpleErrorDisplayInfo>) -> Self {
        Self { at, reason, solution, on_line_and_column, up_to_line_an_column, unexplained_causes, explained_causes }
    }

    /// Returns how many causes it holds, plus itself
    pub(crate) fn complexity(&self) -> usize {
        1 + self.explained_causes.iter().map(|display| display.complexity()).sum::<usize>()
    }

    /// Tells if this error is explained, this is: When at, reason, solution or on_line_and_column
    /// is given (See parameter at [SimpleErrorDisplayInfo]), or when explained_causes isn't empty.
    pub fn is_explained(&self) -> bool {
        self.at.is_some() || self.reason.is_some() || self.solution.is_some() || self.on_line_and_column.is_some() || !self.explained_causes.is_empty()
    }

    /// Gives a string displaying this error, its format is:
    ///
    /// * Position: From which line and column it happens up to which line and column.
    /// * At: String defining where it happened.
    /// * Error: Explanation on why the error happened.
    /// * Has: Count of explained and unexplained errors (Omitted when there is just one explained
    /// error, see [SimpleErrorDisplayInfo::is_explained] for more info.
    /// * Cause/Causes: Repeats this same structure for every explained cause.
    ///
    /// Since all of the fields are optional, if all are empty it returns "Error: Unexplained error"
    /// instead of an empty string.
    pub fn as_display_string(&self) -> String {
        self.__as_display_string(false)
            .unwrap_or_else(|| "Error: Unexplained error".to_string())
    }

    /// Gives a string displaying this error, its format is:
    ///
    /// * Position: From which line and column it happens up to which line and column.
    /// * At: String defining where it happened.
    /// * Error: Explanation on why the error happened.
    /// * Has: Count of explained and unexplained errors (Omitted when there is just one explained
    /// error, see [SimpleErrorDisplayInfo::is_explained] for more info.
    /// * Cause/Causes: Repeats this same structure for every explained cause.
    ///
    /// Since all of the fields are optional, it might return [None].
    fn __as_display_string(&self, is_displaying_as_cause_of_other: bool) -> Option<String> {
        let where_ = &self.at;
        let location = &self.on_line_and_column.map(|(line_of_start, column_of_start)| {
            format!("On line {line_of_start} and column {column_of_start}{}",
                    self.up_to_line_an_column.map(|(line_of_end, column_of_end)|
                        format!(" up to line {line_of_end} and column {column_of_end}")).unwrap_or_default())
        });
        let description = &self.reason.clone().or(Some("Unexplained error".to_string()));
        let solution = &self.solution;

        let explained_causes_count = Some(pluralize(self.unexplained_causes, "unexplained cause", ""))
            .filter(|string| !string.is_empty()).map(|string| string.trim().to_string());
        let unexplained_causes_count = Some(pluralize(self.explained_causes.len(), "explained cause", ""))
            .filter(|string| !string.is_empty()).map(|string| string.trim().to_string());

        let causes_is_just_one_explained = self.explained_causes.len() == 1 && self.unexplained_causes == 0;

        let causes_count = &if causes_is_just_one_explained { None } else if explained_causes_count.is_some() && unexplained_causes_count.is_some() {
            Some(format!("{} and {}", unexplained_causes_count.unwrap(), explained_causes_count.unwrap()))
        } else if explained_causes_count.is_some() || unexplained_causes_count.is_some() {
            Some(format!("{}", explained_causes_count.or(unexplained_causes_count).unwrap()))
        } else {
            None
        }.map(|cause| format!("{cause}."));

        let explained_causes = &match self.explained_causes.len() {
            0 => None,
            1 => Some(self.explained_causes.get(0).unwrap().__as_display_string(true).unwrap()),
            _ => {
                let explained_causes = self.explained_causes.iter().map(|cause| cause.__as_display_string(true))
                    .map(|opt| opt.unwrap())
                    .enumerate()
                    .map(|(cause_no, cause)| format!("- Cause nº {} -\n{cause}", cause_no + 1));
                Some(join_strings("\n\n", explained_causes))
            }
        }.map(|explained_cause| format!("\n{explained_cause}"));

        let causes_prefix = if causes_is_just_one_explained { "Cause" } else { "Causes" };

        let description_lines = [
            ("Position", usize::MAX, location),
            ("At", usize::MAX, where_),
            ("Error", usize::MAX, description),
            ("Solution", usize::MAX, solution),
            ("Has", usize::MAX, causes_count),
            (causes_prefix, 2, explained_causes)
        ]
            .into_iter()
            .filter(|(_, _, contents)| contents.is_some())
            .map(|(prefix, max_ident, contents)| {
                let contents = contents.as_ref().unwrap();
                let prefix = if is_displaying_as_cause_of_other { "- " } else { "" }.to_string() + &prefix.to_string() + ": ";
                let prefixed_contents = format!("{prefix}{contents}");
                let extra_ident_on_causes = if is_displaying_as_cause_of_other { 2 } else { 0 };
                let spaces = prefix.len().min(max_ident.checked_add(extra_ident_on_causes).unwrap_or(usize::MAX));
                let spaced_contents = ident_lines_except_first(prefixed_contents, spaces);
                spaced_contents
            });
        let res = join_strings("\n", description_lines);
        Some(res)
    }
}
