use alloc::sync::Arc;
use core::fmt::Debug;
use alloc::string::String;

use crate::simple_error::SimpleError;
use crate::simple_error_explanation::SimpleErrorExplanation;
use crate::SimpleErrorDisplayInfo;

/// Implementors give a textual explanation on why an error happen and how to solve it through the
/// method [SimpleErrorDetail::explain_error], this means you'll implement this trait for your
/// specific error types.
pub trait SimpleErrorDetail: Debug {
    /// Explains what the happening of this error ([SimpleErrorExplanation::explanation]) and how to
    /// solve it [SimpleErrorExplanation::solution].
    ///
    /// While both explanation and solution are optional, it is expected for implementors to at
    /// least give an explanation, and highly recommended to also give a solution for it.
    fn explain_error(&self) -> SimpleErrorExplanation;

    /// Turns this error value into a [SimpleError] containing both the error itself and the
    /// location it happened at on a certain string, this is specially useful when your error
    /// represents a parsing error.
    fn at<'input>(self, where_: &'input str) -> SimpleError<'input> where Self: Sized + 'input {
        SimpleError::new().error_detail(self).at(where_)
    }

    /// Turns this error value into a [SimpleError] containing both the error itself and start line
    /// and column this error happened, this is specially useful when your error represents a
    /// parsing error.
    fn start_point_of_error<'input>(self, line: usize, column: usize) -> SimpleError<'input> where Self: Sized + 'input {
        SimpleError::new().error_detail(self).end_point_of_error(line, column)
    }

    /// Turns this error value into a [SimpleError] containing both the error itself and the line
    /// and column where this error finishes from happening, this is specially useful when your
    /// error represents a parsing error.
    fn end_point_of_error<'input>(self, line: usize, column: usize) -> SimpleError<'input> where Self: Sized + 'input {
        SimpleError::new().error_detail(self).end_point_of_error(line, column)
    }

    /// Turns this error value into a [SimpleError] the error itself
    fn to_simple_error<'input>(self) -> SimpleError<'input> where Self: Sized + 'input {
        SimpleError::new().error_detail(self)
    }

    /// Turns this error value into a [SimpleError] the error itself and another error which caused
    /// this one.
    fn with_cause<'input, PError: Into<SimpleError<'input>>>(self, cause: PError) -> SimpleError<'input> where Self: Sized + 'input {
        SimpleError::new().error_detail(self).with_cause(cause.into())
    }

    /// Turns this error into a [SimpleErrorDisplayInfo], which will hold at most a reason and a
    /// solution.
    fn to_display_struct(self, colorize: bool) -> SimpleErrorDisplayInfo where Self: Sized {
        SimpleError::new().error_detail(self).as_display_struct(colorize)
    }
}

/// Deref implementation of SimpleErrorDetail for Arc
impl<'lf> SimpleErrorDetail for Arc<dyn SimpleErrorDetail + 'lf> {
    /// Deref implementation of SimpleErrorDetail for Arc
    fn explain_error(&self) -> SimpleErrorExplanation {
        (&**self).explain_error()
    }
}

/// Deref implementation of SimpleErrorDetail for Arc of anything that is [SimpleErrorDetail].
impl<T: SimpleErrorDetail> SimpleErrorDetail for Arc<T> {
    /// Deref implementation of SimpleErrorDetail for Arc of anything that is [SimpleErrorDetail].
    fn explain_error(&self) -> SimpleErrorExplanation {
        (&**self).explain_error()
    }
}

/// SimpleErrorExplanation implements SimpleErrorDetail by giving a copy of itself as an error
/// explanation.
impl<'input> SimpleErrorDetail for SimpleErrorExplanation<'input> {
    /// SimpleErrorExplanation implements SimpleErrorDetail by giving a copy of itself as an error
    /// explanation.
    fn explain_error(&self) -> SimpleErrorExplanation {
        (&*self).clone()
    }
}

/// String can be used as an SimpleErrorExplanation whose explanation is a copy of this String, this
/// is useful if you don't want to create a type for your errors
impl SimpleErrorDetail for String {
    /// String can be used as an SimpleErrorExplanation whose explanation is a copy of this String, this
    /// is useful if you don't want to create a type for your errors
    fn explain_error(&self) -> SimpleErrorExplanation {
        SimpleErrorExplanation::new().explanation(self.clone())
    }
}