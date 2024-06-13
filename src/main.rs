extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Debug, Display, Formatter};
use core::mem;

use colored::{Color, ColoredString, Colorize, Styles};
use itertools::Itertools;

use crate::alloc::string::ToString;

#[derive(Debug)]
struct ParsingError<'input, AstError: ParsingErrorDetail> {
    where_: Option<&'input str>,
    ast_error: Option<AstError>,
    start_point_of_error: Option<(usize, usize)>,
    end_point_of_error: Option<(usize, usize)>,
    causes: Vec<ParsingError<'input, Box<dyn ParsingErrorDetail>>>,
}

impl ParsingErrorDetail for Box<(dyn ParsingErrorDetail + 'static)> {
    fn explain_error(&self) -> ErrorExplanation {
        (&**self).explain_error()
    }
}

impl<T: ParsingErrorDetail> ParsingErrorDetail for Box<T> {
    fn explain_error(&self) -> ErrorExplanation {
        (&**self).explain_error()
    }
}

fn turn_parsing_error_trait_into_box_dyn<'a, Parsing: ParsingErrorDetail + 'a>(parsing: Parsing) -> Box<dyn ParsingErrorDetail + 'a> {
    Box::new(parsing)
}

impl<'input, AstError: ParsingErrorDetail> ParsingError<'input, AstError> {
    fn box_value(self) -> ParsingError<'input, Box<AstError>> {
        ParsingError::<'input, Box<AstError>> {
            where_: self.where_,
            ast_error: self.ast_error.map(|error| Box::new(error)),
            start_point_of_error: self.start_point_of_error,
            end_point_of_error: self.end_point_of_error,
            causes: self.causes,
        }
    }

    pub fn new(ast_error: AstError) -> Self {
        Self { where_: None, ast_error: Some(ast_error), start_point_of_error: None, end_point_of_error: None, causes: Vec::new() }
    }
    pub fn location_str(mut self, location_str: &'input str) -> Self {
        self.where_ = Some(location_str);
        self
    }
    pub fn start_point_of_error(mut self, line: usize, column: usize) -> Self {
        self.start_point_of_error = Some((line, column));
        self
    }
    pub fn end_point_of_error(mut self, line: usize, column: usize) -> Self {
        self.end_point_of_error = Some((line, column));
        self
    }
    pub fn ast_error(mut self, ast_error: AstError) -> Self {
        self.ast_error = Some(ast_error);
        self
    }

    pub fn add_cause<'selflf, Parsing: ParsingErrorDetail + 'static>(&'selflf mut self, cause: ParsingError<'input, Parsing>) {
        let explanation = cause.ast_error.map(|v| turn_parsing_error_trait_into_box_dyn(v));
        self.causes.push(ParsingError {
            where_: cause.where_,
            ast_error: explanation,
            start_point_of_error: cause.start_point_of_error,
            end_point_of_error: cause.end_point_of_error,
            causes: cause.causes,
        });
    }

    fn to_display_string(&self, force_no_colorize: bool) -> String {
        if force_no_colorize { colored::control::set_override(false) }
        let ErrorExplanation
        {
            complete_marker: general_colorizer,
            explanation: error_description,
            colorization_markers: substring_colorizers
        }
            = self.ast_error.as_ref().map(|ast_error| ast_error.explain_error()).unwrap_or_default();
        if force_no_colorize { colored::control::unset_override() }
        let where_ = match self.where_ {
            Some(where_) if !where_.trim().is_empty() => {
                let mut where_string = String::new();
                where_string.push_str("On: ");
                if !force_no_colorize {
                    let where_ = colorize(where_, substring_colorizers, general_colorizer);
                    where_string.push_str(&where_);
                } else {
                    where_string.push_str(where_);
                }
                where_string
            }
            _ => String::new()
        };
        let location = match (self.start_point_of_error, self.end_point_of_error) {
            (Some((line_of_start, column_of_start)), Some((line_of_end, column_of_end))) => {
                "Where: From line ".to_string() + &line_of_start.to_string() + " and column " + &column_of_start.to_string()
                    + " to " + &line_of_end.to_string() + " and column " + &column_of_end.to_string()
            }
            (Some((line_of_start, column_of_start)), _) => {
                "Where: On line ".to_string() + &line_of_start.to_string() + " and column " + &column_of_start.to_string()
            }
            _ => String::new()
        };
        let description = {
            if !error_description.trim().is_empty() {
                "Reason: ".to_string() + &error_description
            } else {
                String::new()
            }
        };
        let causes = {
            let causes = self.causes.iter().map(|cause| cause.to_display_string(force_no_colorize))
                .filter(|cause| !cause.trim().is_empty())
                .join("\n");
            if !causes.trim().is_empty() {
                "Causes:\n".to_string() + &causes
            } else {
                String::new()
            }
        };
        let mut displayed_error = [where_, description, location, causes]
            .into_iter()
            .filter(|string| !string.is_empty())
            .join("\n");
        displayed_error
    }
}

impl<'input, AstError: ParsingErrorDetail> Display for ParsingError<'input, AstError> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.to_display_string(false))
    }
}

impl<'input, AstError: ParsingErrorDetail> std::error::Error for ParsingError<'input, AstError> {}

#[derive(Default)]
pub struct ErrorExplanation<'input> {
    explanation: String,
    complete_marker: Option<Colorization>,
    colorization_markers: Vec<(&'input str, Colorization)>,
}

impl<'input> ErrorExplanation<'input> {
    pub fn new(explanation: String) -> Self {
        Self { explanation, colorization_markers: Vec::new(), complete_marker: None }
    }

    pub fn complete_input_colorization(mut self, complete_marker: Colorization) -> ErrorExplanation<'input> {
        self.complete_marker = Some(complete_marker);
        self
    }

    pub fn colorization_markers(mut self, colorization_markers: Vec<(&'input str, Colorization)>) -> ErrorExplanation<'input> {
        self.colorization_markers.extend(colorization_markers);
        self
    }
}

#[derive(Clone)]
pub struct Colorization {
    foreground: Option<colored::Color>,
    background: Option<colored::Color>,
    style_const: Option<u16>,
}


const STYLES: [Styles; 9] = [Styles::Clear, Styles::Bold, Styles::Dimmed, Styles::Underline,
    Styles::Reversed, Styles::Italic, Styles::Blink, Styles::Hidden, Styles::Strikethrough];

const fn sytle_to_index(style: &Styles) -> usize {
    match style {
        Styles::Clear => 0,
        Styles::Bold => 1,
        Styles::Dimmed => 2,
        Styles::Underline => 3,
        Styles::Reversed => 4,
        Styles::Italic => 5,
        Styles::Blink => 6,
        Styles::Hidden => 7,
        Styles::Strikethrough => 8,
    }
}


impl Colorization {
    pub const fn new() -> Colorization {
        Self { foreground: None, background: None, style_const: None }
    }

    pub const fn foreground(mut self, color: colored::Color) -> Colorization {
        self.foreground = Some(color);
        self
    }

    pub const fn background(mut self, color: colored::Color) -> Colorization {
        self.background = Some(color);
        self
    }

    pub const fn style(mut self, style: Styles) -> Colorization {
        match style {
            Styles::Clear => self.style_const = Some(1 << sytle_to_index(&Styles::Clear)),
            style => {
                if self.style_const.is_none() {
                    self.style_const = Some(0);
                }
                let this_style_const = match self.style_const {
                    None => unreachable!(),
                    Some(style_const) => style_const,
                };
                self.style_const = Some(this_style_const | (1 << sytle_to_index(&style)));
            }
        }
        self
    }

    pub const fn join_const(mut this: Self, other: Self) -> Self {
        if other.foreground.is_some() {
            this.foreground = other.foreground;
        }
        if other.background.is_some() {
            this.background = other.background;
        }
        if this.style_const.is_some() && other.style_const.is_some() {
            let this_style_const = match this.style_const {
                None => { unreachable!() }
                Some(n) => { n }
            };
            let other_style_const = match other.style_const {
                None => { unreachable!() }
                Some(n) => { n }
            };

            let is_clear_style = (other_style_const & (1 << sytle_to_index(&Styles::Clear))) == 1 << sytle_to_index(&Styles::Clear);
            if is_clear_style {
                this.style_const = other.style_const;
                this.foreground = other.foreground;
                this.background = other.background;
            } else {
                this.style_const = Some(this_style_const | other_style_const);
            }
        } else {
            if other.style_const.is_some() {
                this.style_const = other.style_const;
            }
        }
        this
    }

    fn get_styles(&self) -> impl IntoIterator<Item=Styles> + '_ {
        STYLES.into_iter().filter(|style|
            self.style_const.is_some() && (self.style_const.unwrap() & (1 << sytle_to_index(&style))) == 1 << sytle_to_index(&style)
        )
    }

    pub fn styles<StyleT: Into<colored::Styles>, StylesIter: IntoIterator<Item=StyleT>>(mut self, styles: StylesIter) -> Colorization {
        for style in styles {
            self = self.style(style.into());
        }
        self
    }
}


trait ParsingErrorDetail: Debug {
    fn explain_error(&self) -> ErrorExplanation;

    fn location_str<'input>(self, where_: &'input str) -> ParsingError<'input, Self> where Self: Sized {
        ParsingError::new(self).location_str(where_)
    }
    fn start_point_of_error<'input>(self, line: usize, column: usize) -> ParsingError<'input, Self> where Self: Sized {
        ParsingError::new(self).end_point_of_error(line, column)
    }
    fn end_point_of_error<'input>(self, line: usize, column: usize) -> ParsingError<'input, Self> where Self: Sized {
        ParsingError::new(self).end_point_of_error(line, column)
    }

    fn as_parsing_error<'input>(self) -> ParsingError<'input, Self> where Self: Sized {
        ParsingError::new(self)
    }
}

impl<'input, T: ParsingErrorDetail> From<T> for ParsingError<'input, T> {
    fn from(value: T) -> Self {
        ParsingError::new(value)
    }
}


#[derive(Debug)]
enum ASTBuildingError<'input> {
    VariableNotInScope { variable_name: &'input str },
    CannotCompare { value_1: &'input str, comparator: &'input str, value_2: &'input str },
    CannotParsePredicate { predicate: &'input str },
}

const MY_HIDDEN_BACKGROUND: Colorization = Colorization::new()
    .foreground(Color::Blue).style(Styles::Dimmed);

impl<'input> ParsingErrorDetail for ASTBuildingError<'input> {
    fn explain_error(&self) -> ErrorExplanation {
        let explanation;
        let mut color_markers = Vec::new();
        match self {
            ASTBuildingError::VariableNotInScope { variable_name } => {
                explanation = {
                    "The variable ".to_string() + &variable_name.bold() + " does not exist"
                };
                color_markers = alloc::vec![
                    (*variable_name, Colorization::new()
                        .foreground(Color::Red)
                        .styles([Styles::Clear, Styles::Italic, Styles::Bold])
                    ),
                ];
            }
            ASTBuildingError::CannotCompare { value_1, value_2, comparator } => {
                explanation = {
                    "Values ".to_string() + value_1 + " and " + value_2 + " cannot be compared due to different types (Comparator used " + &comparator.italic() + ")"
                };
                color_markers = alloc::vec![
                    (*value_1, Colorization::new()
                        .foreground(Color::Blue)
                        .styles([Styles::Clear, Styles::Italic, Styles::Bold])
                    ),
                    (*value_2, Colorization::new()
                        .foreground(Color::Red)
                        .styles([Styles::Clear, Styles::Italic, Styles::Bold])
                    ),
                ];
            }
            ASTBuildingError::CannotParsePredicate { predicate } => {
                explanation = "This if predicate (".to_string() + &predicate.italic().to_string() + ") doesn't returns a bool value";
                color_markers = alloc::vec![
                    (*predicate, Colorization::new()
                        .foreground(Color::Blue)
                        .styles([Styles::Clear, Styles::Italic, Styles::Bold])
                    ),
                ];
            }
        }
        ErrorExplanation::new(explanation).colorization_markers(color_markers).complete_input_colorization(MY_HIDDEN_BACKGROUND)
    }
}


fn mem_dir_of_string(string: &str) -> (usize, usize) {
    let dir = unsafe { mem::transmute::<_, usize>(string.as_ptr()) };
    (dir, dir + string.len())
}

fn range_contains_other(range_1_start: usize, range_1_end: usize, range_2_start: usize, range_2_end: usize) -> bool {
    let res = range_2_end > range_1_start && range_2_start < range_1_end;
    let res = res;
    res
}

fn main() {
    let input = "if a==1";

    let variable_not_in_scope_error = ASTBuildingError::VariableNotInScope { variable_name: &input[3..4] }.location_str(&input);
    //println!("{variable_not_in_scope_error}");

    let cannot_compare_error = ASTBuildingError::CannotCompare { value_1: &input[3..4], value_2: &input[6..7], comparator: &input[4..6] }.location_str(&input[3..7]);
    //println!("{cannot_compare_error}");

    let mut cannot_parse_predicate = ASTBuildingError::CannotParsePredicate { predicate: &input[3..7] }.location_str(&input);

    cannot_parse_predicate.add_cause(cannot_compare_error.box_value());
    cannot_parse_predicate.add_cause(variable_not_in_scope_error.box_value());
    println!("{cannot_parse_predicate}");


    /*
    let _ = std::fs::write(r"D:\error.color", error.to_display_string(false));


    let input_mo = vec![
        (&input[0..7], Colorization::new().foreground(Color::TrueColor { r: 255, g: 0, b: 0 })),
        (&input[1..7], Colorization::new().foreground(Color::TrueColor { r: 255, g: 255, b: 0 })),
        (&input[2..7], Colorization::new().foreground(Color::TrueColor { r: 0, g: 255, b: 0 })),
        (&input[3..7], Colorization::new().foreground(Color::TrueColor { r: 0, g: 255, b: 255 })),
        (&input[4..7], Colorization::new().foreground(Color::TrueColor { r: 0, g: 0, b: 255 })),
        (&input[5..7], Colorization::new().foreground(Color::TrueColor { r: 255, g: 0, b: 255 })),
        (&input[6..7], Colorization::new().foreground(Color::TrueColor { r: 0, g: 0, b: 0 })),
    ];

    let input = apply_substring_transformations(input, input_mo, None);
    println!("{input}");
    */
}

fn colorize<'input>(input: &'input str, mut input_modifiers: Vec<(&'input str, Colorization)>, general_colorization: Option<Colorization>) -> String {
    let (input_start, input_end) = mem_dir_of_string(input);
    let input_len = input.len();
    if let Some(general_colorization) = general_colorization {
        input_modifiers.insert(0, (&input, general_colorization));
    }

    let ranges_and_modifiers = input_modifiers
        .iter()
        .map(|(str_slice, value)| {
            let (offset_start, offset_end) = mem_dir_of_string(str_slice);
            (offset_start, offset_end, value)
        })
        .filter(|(offset_start, offset_end, _)| {
            range_contains_other(*offset_start, *offset_end, input_start, input_end)
        })
        .map(|(offset_start, offset_end, value)| {
            (
                offset_start.checked_sub(input_start).unwrap_or(0).min(input_len),
                offset_end.checked_sub(input_start).unwrap_or(0).min(input_len),
                value
            )
        })
        .filter(|(start, end, _)| end > start)
        .collect::<Vec<_>>();

    let bounds = ranges_and_modifiers
        .iter()
        .flat_map(|(start, end, _)| [*start, *end])
        .sorted()
        .dedup()
        .collect::<Vec<_>>();

    let ranges_and_modifiers =
        bounds.windows(2)
            .map(|ran| (ran[0], ran[1]))
            .map(|(start, end)| {
                let mut colorization = Colorization::new();
                for found_colorizer in ranges_and_modifiers.iter().filter(|range_and_modifier|
                    range_contains_other(start, end, range_and_modifier.0, range_and_modifier.1))
                    .map(|(_, _, modifier)| modifier) {
                    colorization = Colorization::join_const(colorization, (*found_colorizer).clone());
                }
                (start, end, colorization)
            })
            .collect::<Vec<_>>();


    let mut input = input.to_string();
    ranges_and_modifiers.into_iter()
        .sorted_by(|(start_1, _, _), (start_2, _, _)| start_1.cmp(start_2).reverse())
        .for_each(|(start, offset_end, modifier)| {
            let mut modified = input[start..offset_end].to_string();
            if let Some(background_color) = modifier.background {
                modified = modified.on_color(background_color).to_string();
            }
            if let Some(foreground_color) = modifier.foreground {
                modified = modified.color(foreground_color).to_string();
            }
            for style in modifier.get_styles() {
                let stylizer: fn(ColoredString) -> ColoredString = match style {
                    Styles::Clear => Colorize::clear,
                    Styles::Bold => Colorize::bold,
                    Styles::Dimmed => Colorize::dimmed,
                    Styles::Underline => Colorize::underline,
                    Styles::Reversed => Colorize::reversed,
                    Styles::Italic => Colorize::italic,
                    Styles::Blink => Colorize::blink,
                    Styles::Hidden => Colorize::hidden,
                    Styles::Strikethrough => Colorize::strikethrough,
                };
                modified = stylizer(ColoredString::from(modified)).to_string().to_string();
            }
            let mut res = String::new();
            res.push_str(&input[..start]);
            res.push_str(&modified);
            res.push_str(&input[offset_end..]);
            input = res;
        });
    input
}