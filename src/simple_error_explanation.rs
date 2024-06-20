use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Holds a possible explanation and solution for an error, and for parsing errors it also holds a
/// [Colorizer] for colorizing both the whole and parts of the input indicated on [SimpleError::at].
#[derive(Default)]
pub struct SimpleErrorExplanation<'input> {
    pub(crate) explanation: Option<String>,
    pub(crate) solution: Option<String>,
    #[cfg(feature = "colorization")]
    pub(crate) whole_marker: Option<string_colorization::Colorizer>,
    #[cfg(feature = "colorization")]
    pub(crate) colorization_markers: Vec<(&'input str, string_colorization::Colorizer)>,
    #[cfg(not(feature = "colorization"))]
    _input_lifetime: core::marker::PhantomData<&'input ()>
}

impl<'input> SimpleErrorExplanation<'input> {

    /// Creates a new empty [SimpleErrorExplanation]
    pub fn new() -> Self {
        #[cfg(feature = "colorization")]
        let res = Self { explanation: None, solution: None, colorization_markers: Vec::new(), whole_marker: None };
        #[cfg(not(feature = "colorization"))]
        let res = Self { explanation: None, solution: None, _input_lifetime: Default::default() };
        res
    }

    /// Adds an explanation on why this error happened, like 'Variable ***my_missing_variable*** was
    /// not found'.
    pub fn explanation<Str:Into<String>>(mut self, explanation: Str) -> Self {
        self.explanation = Some(explanation.into())
            .map(|explanation| explanation.trim().to_string())
            .filter(|explanation| !explanation.is_empty());
        self
    }

    /// Adds a solution on how to solve this error, like 'Create variable ***my_missing_variable***
    /// before using it'.
    pub fn solution<Str:Into<String>>(mut self, solution: Str) -> Self {
        self.solution = Some(solution.into())
            .map(|solution| solution.trim().to_string())
            .filter(|solution| !solution.is_empty());
        self
    }

    #[cfg(feature = "colorization")]
    /// Marker for colorizing the whole input indicated at [SimpleError::at], this is used on
    /// parsing errors.
    pub const fn whole_input_colorization(mut self, complete_marker: string_colorization::Colorizer) -> Self {
        self.whole_marker = Some(complete_marker);
        self
    }

    #[cfg(feature = "colorization")]
    /// Markers for colorizing the substrings belonging to the input indicated at [SimpleError::at],
    /// this is used on parsing errors, but it requires the substring are references taken from the
    /// same input indicated on [SimpleError::at], or else, they won't get colorized following
    /// [string_colorization::colorize] restrictions.
    pub fn colorization_markers<Color, Input, MarkerIterator>(mut self, colorization_markers: MarkerIterator) -> Self
        where Color: Into<string_colorization::Colorizer>,
              Input: Into<&'input str>,
              MarkerIterator: IntoIterator<Item=(Input, Color)> {
        self.colorization_markers.extend(colorization_markers.into_iter().map(|(input, color)| (input.into(), color.into())));
        self
    }

    #[cfg(feature = "colorization")]
    /// Marker for colorizing the substrings belonging to the input indicated at [SimpleError::at],
    /// this is used on parsing errors, but it requires the substring are references taken from the
    /// same input indicated on [SimpleError::at], or else, they won't get colorized following
    /// [string_colorization::colorize] restrictions.
    pub fn colorization_marker(mut self, string: &'input str, colorization: string_colorization::Colorizer) -> Self {
        self.colorization_markers.push((string, colorization));
        self
    }
}
