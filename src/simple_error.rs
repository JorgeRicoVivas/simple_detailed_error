use alloc::collections::VecDeque;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};

use crate::simple_error_detail::SimpleErrorDetail;
use crate::simple_error_display_info::SimpleErrorDisplayInfo;
use crate::simple_error_explanation::SimpleErrorExplanation;

/// Holds information to explain an error, such as its detail (What happened and how to solve it),
/// what errors caused this error, or for parsing errors, at which lines and column did they start /
/// end, and a &str to where it happened.
#[derive(Debug, Default, Clone)]
pub struct SimpleError<'input> {
    where_: Option<&'input str>,
    error_detail: Option<Arc<dyn SimpleErrorDetail + 'input>>,
    start_point_of_error: Option<(usize, usize)>,
    end_point_of_error: Option<(usize, usize)>,
    causes: Vec<SimpleError<'input>>,
}

/// Creates a SimpleError whose details is this owned value implementing SimpleErrorDetail
impl<'input, T: SimpleErrorDetail + 'input> From<T> for SimpleError<'input> {
    /// Creates a SimpleError whose details is this value
    fn from(value: T) -> Self {
        SimpleError::new().error_detail(value)
    }
}

/// This is only implemented when using the std feature, enabled by default.
///
/// SimpleError implements [std::error::Error] as it also implements [Display] and [Debug].
#[cfg(feature = "std")]
impl<'input> std::error::Error for SimpleError<'input> {}

/// SimpleErrors can display through the [SimpleErrorDisplayInfo] struct calling to
/// [SimpleErrorDisplayInfo::as_display_string].
impl<'input> Display for SimpleError<'input> {
    /// SimpleErrors can display through the [SimpleErrorDisplayInfo] struct calling to
    /// [SimpleErrorDisplayInfo::as_display_string].
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.as_display_struct(true).as_display_string())
    }
}

impl<'input> SimpleError<'input> {
    /// Creates a new SimpleError where no information is given.
    pub fn new() -> Self {
        Self { where_: None, error_detail: None, start_point_of_error: None, end_point_of_error: None, causes: Vec::new() }
    }

    /// Responds to: What and how to solve it.
    ///
    /// Indicates the error detail for this error
    pub fn error_detail<ErrorDetail: SimpleErrorDetail + 'input>(mut self, error_detail: ErrorDetail) -> Self {
        self.error_detail = Some(Arc::new(error_detail));
        self
    }

    /// Responds to: Where did it happen, usually on parsing errors.
    ///
    /// Adds a referenced string to show where the error happened, for example 'At: let a = ...'.
    pub fn at(mut self, location_str: &'input str) -> Self {
        self.where_ = Some(location_str);
        self
    }

    /// Responds to: Where does this error starts to happen, usually on parsing errors.
    ///
    /// For example: 'From line 3 and column 5'.
    pub fn start_point_of_error(mut self, line: usize, column: usize) -> Self {
        self.start_point_of_error = Some((line, column));
        self
    }

    /// Responds to: Where does this error finishes from happening, usually on parsing errors.
    ///
    /// For example: 'From line 3 and column 5 **up to line 7 and column 9**'.
    pub fn end_point_of_error(mut self, line: usize, column: usize) -> Self {
        self.end_point_of_error = Some((line, column));
        self
    }

    /// Responds to: Why did it happen.
    ///
    /// Adds an error that caused this one to happen.
    pub fn with_cause<PError: Into<SimpleError<'input>>>(mut self, cause: PError) -> Self {
        self.add_cause(cause.into());
        self
    }

    /// Removes all the causes on why this error happened.
    pub fn without_causes(mut self) -> Self {
        self.causes = Vec::new();
        self
    }

    /// Responds to: Why did it happen.
    ///
    /// Adds an error that caused this one to happen.
    pub fn add_cause<PError: Into<SimpleError<'input>>>(&mut self, cause: PError) {
        self.causes.push(cause.into());
    }

    fn __as_display_struct(&self) -> SimpleErrorDisplayInfo {
        let error_explanation =  self.error_detail.as_ref()
            .map(|error_detail|error_detail.explain_error())
            .unwrap_or_default();

        #[cfg(feature = "colorization")]
        let SimpleErrorExplanation { whole_marker: general_colorizer, explanation: error_description, solution, colorization_markers: substring_colorizers } = error_explanation;
        #[cfg(not(feature = "colorization"))]
        let SimpleErrorExplanation { explanation: error_description, solution, .. } = error_explanation;


        #[cfg(feature = "colorization")]
            let where_ = self.where_.clone()
            .map(|where_| string_colorization::colorize(where_, general_colorizer, substring_colorizers))
            .filter(|string| !string.is_empty()).map(|string| string.trim().to_string());
        #[cfg(not(feature = "colorization"))]
            let where_ = self.where_.clone()
            .filter(|string| !string.is_empty()).map(|string| string.trim().to_string());


        let mut unexplained_causes = 0;
        let mut explained_causes = self.causes.iter()
            .map(|cause| cause.__as_display_struct())
            .filter(|cause| {
                let is_explained = cause.is_explained();
                if !is_explained { unexplained_causes += 1 };
                is_explained
            })
            .collect::<Vec<_>>();
        explained_causes.sort_by_key(|error| error.complexity());

        SimpleErrorDisplayInfo::new(where_, error_description, solution,
                                    self.start_point_of_error.clone(), self.end_point_of_error.clone(), unexplained_causes, explained_causes)
    }

    /// Turns this SimpleError into a [SimpleErrorDisplayInfo], the string might have terminal color
    /// if indicated on the variable *colorize*.
    ///
    /// This struct is a serializable one when the feature 'serde' is enabled, making it useful to
    /// share errors through platforms and also save them for future auditing.
    pub fn as_display_struct(&self, colorize: bool) -> SimpleErrorDisplayInfo {
        #[cfg(feature = "colorization")]
            let forced_no_colorization = !colorize && colored::control::SHOULD_COLORIZE.should_colorize();
        #[cfg(feature = "colorization")]
        if forced_no_colorization {
            colored::control::SHOULD_COLORIZE.set_override(false);
        }
        let res = self.__as_display_struct();
        #[cfg(feature = "colorization")]
        if forced_no_colorization {
            colored::control::SHOULD_COLORIZE.set_override(true);
        }
        res
    }

    /// Returns all the errors not holding any other errors, for example, if we had four errors A B
    /// C and D, where A had B and C as causes, and C had D as cause, it would return B and D as
    /// leaf errors as they are the only errors in the error tree not having any sub-errors /
    /// causes.
    pub fn leaf_errors(&self) -> Vec<&Self> {
        if self.causes.is_empty() {
            return vec![&self];
        } else {
            self.causes.iter().flat_map(|cause| cause.leaf_errors()).collect()
        }
    }

    /// In case this error represents an error tree, it returns every leaf error itself, and as
    /// causes of these errors they have their respective ancestors, for example, if we had four
    /// errors A B C and D, where A had B and C as causes, and C had D as cause, then the leafs
    /// would be B and D, for B it returns it with A as a cause, and for D it returns it with C as a
    /// cause, which also will have A as a cause.
    pub fn inverted_error_tree(&self) -> Vec<SimpleError<'input>> {
        let mut result = Default::default();
        let mut current_stack = Default::default();
        self.revese_errors_int(&mut result, &mut current_stack);

        result.into_iter().map(|mut errors_stack| {
            let mut reverse_error = errors_stack.remove(errors_stack.len() - 1).clone().without_causes();
            while !errors_stack.is_empty() {
                reverse_error = errors_stack.remove(errors_stack.len() - 1).clone().without_causes().with_cause(reverse_error);
            }
            reverse_error
        }).collect::<Vec<_>>()
    }

    fn revese_errors_int<'selflf>(&'selflf self, result: &mut Vec<Vec<&'selflf Self>>, current_stack: &mut VecDeque<&'selflf Self>) {
        current_stack.push_front(self);
        if self.causes.is_empty() {
            result.push(current_stack.clone().into_iter().collect());
        } else {
            self.causes.iter().for_each(|cause| cause.revese_errors_int(result, current_stack));
        }
        current_stack.pop_front();
    }
}
