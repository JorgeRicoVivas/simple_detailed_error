#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! [![crates.io](https://img.shields.io/crates/v/simple_detailed_error.svg)](https://crates.io/crates/simple_detailed_error)
//! [![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/JorgeRicoVivas/simple_detailed_error/rust.yml)](https://github.com/JorgeRicoVivas/simple_detailed_error/actions)
//! [![docs.rs](https://img.shields.io/docs.rs/simple_detailed_error)](https://docs.rs/simple_detailed_error/latest/simple_detailed_error/)
//! [![GitHub License](https://img.shields.io/github/license/JorgeRicoVivas/simple_detailed_error)](https://github.com/JorgeRicoVivas/simple_detailed_error?tab=CC0-1.0-1-ov-file)
//! > *You are reading the documentation for simple_detailed_error version 1.0.0*
//!
//! This crate helps you creating errors by giving you the [SimpleErrorDetail] trait where you give
//! text indicating why an error happens and how to solve it, while still using a pattern that
//! easily allows you to tell the user information about said error, such as what happened, why,
//! how, where, how to solve it and its causes.
//! <br>
//!
//! # Guided and deep example with parsing errors of a scripting language
//!
//! Say we are creating a scripting language inside rust where we receive a script like
//! ```if missing_variable > 0 { return missing_function(missing_variable); }```, this script has
//! two errors: The variable ***missing_variable*** doesn't exists, and the function
//! ***missing_function*** doesn't exist either, in this situation, we would like to show the user
//! an error message like this: <br>
//!
//! \- Error: Couldn't compile code.<br>
//! \- Has: 2 explained causes.<br>
//! \- Causes:<br>
//! &nbsp;&nbsp;&nbsp;&nbsp;- Cause nº 1 -<br>
//! &nbsp;&nbsp;&nbsp;&nbsp;\- At: if missing_variable > 0<br>
//! &nbsp;&nbsp;&nbsp;&nbsp;\- Error: Variable missing_variable doesn't exists.<br>
//! &nbsp;&nbsp;&nbsp;&nbsp;\- Solution: Declare it before using it, like this:<br>
//! &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; let missing_variable = your value<br>
//!  <br>
//! &nbsp;&nbsp;&nbsp;&nbsp;- Cause nº 2 -<br>
//! &nbsp;&nbsp;&nbsp;&nbsp;\- At: return missing_function(missing_variable);<br>
//! &nbsp;&nbsp;&nbsp;&nbsp;\- Error: Function missing_function doesn't exists.<br>
//! &nbsp;&nbsp;&nbsp;&nbsp;\- Solution: Implement an missing_function function, like this:<br>
//! &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; fn missing_function(...) { ...your code here... }<br>
//!
//! <br>
//!
//! ### Defining structs that explain errors
//!
//! For now, let's declare an enum that would represent all the errors we want (They can be
//! different structs, but to make it comfortable, we are going to use three variants), for example:
//!
//! ``` rust
//! #[derive(Debug)]
//! enum CompilationError<'code_input>{
//!     MissingVariable { variable_name : &'code_input str }, // This represents when a non-existing
//!                                                           // variable is referenced
//!     MissingFunction { function_name : &'code_input str }, // This represents when a non-declared
//!                                                           // function tries to get called
//!     RootCompilationError // This isn't really an error, is just one representation for saying
//!                          // 'hey, there are errors in your compilation'
//! }
//! ```
//!
//! Now let's just implement [SimpleErrorDetail] on this enum, this makes it so we have to implement
//! a function where we return a [SimpleErrorExplanation] where we can give an explanation and a
//! solution for the error using the functions [SimpleErrorExplanation::solution] and
//! [SimpleErrorExplanation::explanation] (Note: You are not forced to give neither the solution nor
//! the explanation, but is is highly advised, as they should help your users):
//!
//! ``` rust
//! use simple_detailed_error::{SimpleErrorExplanation, SimpleErrorDetail};
//!
//! impl <'code_input> SimpleErrorDetail for CompilationError<'code_input>{
//!     fn explain_error(&self) -> SimpleErrorExplanation {
//!         match self{
//!             CompilationError::MissingVariable{ variable_name } => {
//!                 SimpleErrorExplanation::new()
//!                     .explanation(format!("Variable {variable_name} doesn't exists."))
//!                     .solution(format!("Declare it before using it, like this:\nlet {variable_name} = *your value*"))
//!             }
//!             CompilationError::MissingFunction{ function_name } => {
//!                 SimpleErrorExplanation::new()
//!                     .explanation(format!("Function {function_name} doesn't exists."))
//!                     .solution(format!("Implement an is_odd function, like this:\nfn {function_name}(...) {{ ...your code here... }}"))
//!             }
//!             CompilationError::RootCompilationError => {
//!                 SimpleErrorExplanation::new().explanation("Couldn't compile code.")
//!             }
//!         }
//!     }
//! }
//!
//! #[derive(Debug)]
//! enum CompilationError<'code_input>{
//!     MissingVariable {variable_name : &'code_input str},
//!     MissingFunction {function_name : &'code_input str},
//!     RootCompilationError
//! }
//! ```
//!
//! <br>
//!
//! ### Creating error values and displaying them
//!
//! Perfect! With this, our enum representing our errors now can use functions like
//! [SimpleErrorDetail::to_parsing_error], which turns our variant into a struct of [SimpleError]
//! containing said variant and using is as a representation of an error whose explanation and
//! solutions are those said when we implemented [SimpleErrorDetail::explain_error].
//!
//! The [SimpleError] struct is one that holds information about an error, such as why it happened,
//! how to solve it, or where it happened, it can also hold other [SimpleError]s inside, this
//! represents an error being caused by another error, or even by multiple errors, for example, we
//! can add an error using [SimpleError::add_cause] or [SimpleError::with_cause].
//!
//! For now, we are going to create a variant of `CompilationError::RootCompilationError` which will
//! hold our errors, and then we are going to stack it with the missing variable and the missing
//! function errors:
//!
//! - Creating the <i>missing variable error</i>: This is simply constructing a
//! `CompilationError::MissingVariable` variant, where the variable name is just a reference to where
//! it says 'missing_variable' in the original input, and since this is a parsing error, we can also
//! use the [SimpleError::at] to indicate a bigger string where the error is happening, we will use
//! it to reference 'if missing_variable > 0'.<br><br>
//! ...Wait, we are using the `at` function, but that's implemented for [SimpleError], why is it
//! working then? Well, the trait [SimpleErrorDetail] implements plenty of functions of
//! [SimpleError], this is so you can use your struct (In this case `CompilationError`) as you were
//! using a value of type [SimpleError].<br><br>
//! - Creating the <i>missing function error</i>: This isn't very different from our previous case,
//! we just constructing a `CompilationError::MissingFunction`, as for the function `at`, this time
//! we will reference 'return missing_function(missing_variable);'.<br><br>
//! - Creating the <i>Error root</i>: This is something you'll perhaps never do in a real project,
//! but we are going to use a base error, in this case  `CompilationError::RootCompilationError`
//! where we will stack the errors using the [SimpleError::with_cause] function, stacking the
//! missing variable and missing function errors, although this is just for showing it's
//! functionality, you should just stack real causes.
//!
//! Once done, we can just can print our [SimpleError] and it will result in the error stack shown
//! earlier.
//!
//! ``` rust
//! use simple_detailed_error::{SimpleErrorExplanation, SimpleErrorDetail};
//!
//! let code_to_compile = "if missing_variable > 0 { return missing_function(missing_variable); }";
//! let missing_variable_error = CompilationError::MissingVariable {variable_name: &code_to_compile[3..19] }
//!                                  .at(&code_to_compile[0..23]);
//! let missing_function_error = CompilationError::MissingFunction {function_name: &code_to_compile[33..49] }
//!                                  .at(&code_to_compile[26..68]);
//! let errors_stacker = CompilationError::RootCompilationError
//!                         .with_cause(missing_variable_error).with_cause(missing_function_error);
//! assert_eq!(format!("{errors_stacker}"), "Error: Couldn't compile code.\nHas: 2 explained causes.\nCauses: \n  - Cause nº 1 -\n  - At: if missing_variable > 0\n  - Error: Variable missing_variable doesn't exists.\n  - Solution: Declare it before using it, like this:\n              let missing_variable = *your value*\n  \n  - Cause nº 2 -\n  - At: return missing_function(missing_variable);\n  - Error: Function missing_function doesn't exists.\n  - Solution: Implement an is_odd function, like this:\n              fn missing_function(...) { ...your code here... }");
//!
//! impl <'code_input> SimpleErrorDetail for CompilationError<'code_input>{
//!     fn explain_error(&self) -> SimpleErrorExplanation {
//!         match self{
//!             CompilationError::MissingVariable{ variable_name } => {
//!                 SimpleErrorExplanation::new()
//!                     .explanation(format!("Variable {variable_name} doesn't exists."))
//!                     .solution(format!("Declare it before using it, like this:\nlet {variable_name} = *your value*"))
//!             }
//!             CompilationError::MissingFunction{ function_name } => {
//!                 SimpleErrorExplanation::new()
//!                     .explanation(format!("Function {function_name} doesn't exists."))
//!                     .solution(format!("Implement an is_odd function, like this:\nfn {function_name}(...) {{ ...your code here... }}"))
//!             }
//!             CompilationError::RootCompilationError => {
//!                 SimpleErrorExplanation::new().explanation("Couldn't compile code.")
//!             }
//!         }
//!     }
//! }
//!
//! #[derive(Debug)]
//! enum CompilationError<'code_input>{
//!     MissingVariable {variable_name : &'code_input str},
//!     MissingFunction {function_name : &'code_input str},
//!     RootCompilationError
//! }
//! ```
//!
//! ### Feature 'colorization': Adding color and emphasis on errors
//!
//! That was quite alright! But... the output could benefit from some colorization, what if the
//! result looked more like this?
//!
//! \- Error: Couldn't compile code.<br>
//! \- Has: 2 explained causes.<br>
//! \- Causes:<br>
//! &nbsp;&nbsp;&nbsp;&nbsp;- Cause nº 1 -<br>
//! &nbsp;&nbsp;&nbsp;&nbsp;</span>\- At: <i style="color:lightblue;opacity:.7;">if </i><span style="color: red;">**missing_variable**</span> <i style="color:lightblue;opacity:.7;">> 0</i><br>
//! &nbsp;&nbsp;&nbsp;&nbsp;\- Error: Variable **missing_variable** doesn't exists.<br>
//! &nbsp;&nbsp;&nbsp;&nbsp;\- Solution: Declare it before using it, like this:<br>
//! &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; let <span style="color:green">missing_variable = <i>*your value*</i><br>
//!  <br>
//! &nbsp;&nbsp;&nbsp;&nbsp;- Cause nº 2 -<br>
//! &nbsp;&nbsp;&nbsp;&nbsp;</span>\- At: <i style="color:lightblue;opacity:.7;">return </i>**<span style="color: red;">missing_function**<i style="color:lightblue;opacity:.7;">(missing_variable);</i><br>
//! &nbsp;&nbsp;&nbsp;&nbsp;\- Error: Function **missing_function** doesn't exists.<br>
//! &nbsp;&nbsp;&nbsp;&nbsp;\- Solution: Declare it before using it, like this:<br>
//! &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; fn <span style="color:green">missing_function (...) { ...*your code here*... }<br>
//!
//! By dimming irrelevant parts and making the parts where the errors happen to be easier to see
//! might direct the attention of the user to the error, this makes them to read less in plenty of
//! times while they still all the relevant information.
//!
//! - Colorizing explanations and solutions: With the [colored] crate you can colorize the
//! explanations and solutions, for example, in `Variable {variable_name} doesn't exist` it would
//! be great if we could turn the variable name into a bold and red name, taking the user's
//! attention directly to it, like 'Variable <span style="color:red">**missing variable**</span>
//! doesn't exist'.<br><br>
//! For this, we can just replace
//! `.explanation(format!("Variable {variable_name} doesn't exists."))` for
//! `.explanation(format!("Variable {} doesn't exists.", variable_name.red().bold()))`.<br><br>
//! - Colorizing the input: The [string_colorization::colorize] takes an &str and then applies some
//! colors and stylizations to other &str of it that we tell, this means that if we had a &str that
//! is part of the &str taken as input, then we could colorize it!<br>
//! For example, in out missing variable error, we used [SimpleError::at], where the substring
//! referenced is 'if missing_variable > 0', thing is, our missing variable variant has another
//! substring whose value is 'missing_variable', this means that if we used
//! [SimpleErrorExplanation::colorization_marker] like this `SimpleErrorExplanation::new()
//! .colorization_marker(variable_name, foreground::Red + style::Bold)`, then 'if missing_variable >
//! 0' would look like 'if <span style="color:red">**missing_variable**</span> > 0'.<br><br>
//! The down-side of this is that your struct or enum must have references to the original source,
//! although this is often the case for many situations, like parsing values as in this situation.
//! <br><br>
//! We can also use [SimpleErrorExplanation::whole_input_colorization] to colorize everything
//! written inside `at`, for example, `SimpleErrorExplanation::new()
//! .complete_input_colorization(foreground::Blue + style::Italic + style::Dimmed)` will make it so
//! `if missing_variable > 0` looks blue, italic and dimmed, telling the user it is not important.
//! <br><br>But... now 'if missing_variable > 0' looks like <i style="color:lightblue;opacity:.7;">
//! if **<span style="color: red;">missing_variable </span>** > 0</i>, not like
//! <i style="color:lightblue;opacity:.7;">if </i>**<span style="color: red;">missing_variable
//! </span>** <i style="color:lightblue;opacity:.7;">> 0</i>, why is 'missing_variable' also dimmed?
//! This is because the complete_input_colorization set it to dim, while the colorization_marker
//! didn't override this, to avoid this, we can use [string_colorization::style::Clear] to remove
//! all the stylization from complete_input_colorization, this makes it so
//! `SimpleErrorExplanation::new() .colorization_marker(variable_name, style::Clear +
//! foreground::Red + style::Bold) .complete_input_colorization(foreground::Blue + style::Italic +
//! style::Dimmed)` while turn 'if missing_variable > 0' into the desired
//! <i style="color:lightblue;opacity:.7;">if </i>**<span style="color: red;">missing_variable
//! </span>** <i style="color:lightblue;opacity:.7;">> 0</i>.
//!
//! ``` rust
//! use colored::Colorize;
//! use string_colorization::{foreground, style};
//! use simple_detailed_error::{SimpleErrorDetail, SimpleErrorExplanation};
//!
//! impl <'code_input> SimpleErrorDetail for CompilationError<'code_input>{
//!     fn explain_error(&self) -> SimpleErrorExplanation {
//!         match self{
//!             CompilationError::MissingVariable{ variable_name } => {
//!                 SimpleErrorExplanation::new()
//!                     .colorization_marker(variable_name, style::Clear + foreground::Red + style::Bold)
//!                     .whole_input_colorization(foreground::Blue + style::Italic + style::Dimmed)
//!                     .explanation(format!("Variable {} doesn't exists.", variable_name.red().bold()))
//!                     .solution(format!("Declare it before using it, like this:\nlet {} = {}", variable_name.green(), "*your value*".italic()))
//!             }
//!             CompilationError::MissingFunction{ function_name } => {
//!                 SimpleErrorExplanation::new()
//!                     .colorization_marker(function_name, style::Clear + foreground::Red + style::Bold)
//!                     .whole_input_colorization(foreground::Blue + style::Italic + style::Dimmed)
//!                     .explanation(format!("Function {} doesn't exists.", function_name.red().bold()))
//!                     .solution(format!("Implement an {function_name} function, like this:\nfn {}(...) {{ ...{}... }}", function_name.green(), "*your code here*".italic() ))
//!             }
//!             CompilationError::RootCompilationError => {
//!                 SimpleErrorExplanation::new().explanation("Couldn't compile code.")
//!             }
//!         }
//!     }
//! }
//!
//! #[derive(Debug)]
//! enum CompilationError<'code_input>{
//!     MissingVariable {variable_name : &'code_input str},
//!     MissingFunction {function_name : &'code_input str},
//!     RootCompilationError,
//! }
//! ```
//!
//! Great! Now everything is finished! This is the resulting code:
//!
//! ``` rust
//! use colored::Colorize;
//! use string_colorization::{foreground, style};
//! use simple_detailed_error::{SimpleErrorExplanation, SimpleErrorDetail};
//!
//! colored::control::set_override(true); // This forces the colorization to be applied, this should
//!                                       // not appear in your code, is written here to force it
//!                                       // for testing purposes to show you this code is correct.
//!
//! let code_to_compile = "if missing_variable > 0 { return missing_function(missing_variable); }";
//! let missing_variable_error = CompilationError::MissingVariable {variable_name: &code_to_compile[3..19] }
//!                                  .at(&code_to_compile[0..23]);
//! let missing_function_error = CompilationError::MissingFunction {function_name: &code_to_compile[33..49] }
//!                                  .at(&code_to_compile[26..68]);
//! let errors_stacker = CompilationError::RootCompilationError
//!                         .with_cause(missing_variable_error).with_cause(missing_function_error);
//! assert_eq!(format!("{errors_stacker}"), "Error: Couldn't compile code.\nHas: 2 explained causes.\nCauses: \n  - Cause nº 1 -\n  - At: \u{1b}[34m\u{1b}[3m\u{1b}[2mif \u{1b}[0m\u{1b}[34m\u{1b}[3m\u{1b}[0m\u{1b}[34m\u{1b}[0m\u{1b}[31m\u{1b}[1mmissing_variable\u{1b}[0m\u{1b}[31m\u{1b}[0m\u{1b}[34m\u{1b}[3m\u{1b}[2m > 0\u{1b}[0m\u{1b}[34m\u{1b}[3m\u{1b}[0m\u{1b}[34m\u{1b}[0m\n  - Error: Variable \u{1b}[1;31mmissing_variable\u{1b}[0m doesn't exists.\n  - Solution: Declare it before using it, like this:\n              let \u{1b}[32mmissing_variable\u{1b}[0m = \u{1b}[3m*your value*\u{1b}[0m\n  \n  - Cause nº 2 -\n  - At: \u{1b}[34m\u{1b}[3m\u{1b}[2mreturn \u{1b}[0m\u{1b}[34m\u{1b}[3m\u{1b}[0m\u{1b}[34m\u{1b}[0m\u{1b}[31m\u{1b}[1mmissing_function\u{1b}[0m\u{1b}[31m\u{1b}[0m\u{1b}[34m\u{1b}[3m\u{1b}[2m(missing_variable);\u{1b}[0m\u{1b}[34m\u{1b}[3m\u{1b}[0m\u{1b}[34m\u{1b}[0m\n  - Error: Function \u{1b}[1;31mmissing_function\u{1b}[0m doesn't exists.\n  - Solution: Implement an missing_function function, like this:\n              fn \u{1b}[32mmissing_function\u{1b}[0m(...) { ...\u{1b}[3m*your code here*\u{1b}[0m... }");
//!
//! impl <'code_input> SimpleErrorDetail for CompilationError<'code_input>{
//!     fn explain_error(&self) -> SimpleErrorExplanation {
//!         match self{
//!             CompilationError::MissingVariable{ variable_name } => {
//!                 SimpleErrorExplanation::new()
//!                     .colorization_marker(variable_name, style::Clear + foreground::Red + style::Bold)
//!                     .whole_input_colorization(foreground::Blue + style::Italic + style::Dimmed)
//!                     .explanation(format!("Variable {} doesn't exists.", variable_name.red().bold()))
//!                     .solution(format!("Declare it before using it, like this:\nlet {} = {}", variable_name.green(), "*your value*".italic()))
//!             }
//!             CompilationError::MissingFunction{ function_name } => {
//!                 SimpleErrorExplanation::new()
//!                     .colorization_marker(function_name, style::Clear + foreground::Red + style::Bold)
//!                     .whole_input_colorization(foreground::Blue + style::Italic + style::Dimmed)
//!                     .explanation(format!("Function {} doesn't exists.", function_name.red().bold()))
//!                     .solution(format!("Implement an {function_name} function, like this:\nfn {}(...) {{ ...{}... }}", function_name.green(), "*your code here*".italic() ))
//!             }
//!             CompilationError::RootCompilationError => {
//!                 SimpleErrorExplanation::new().explanation("Couldn't compile code.")
//!             }
//!         }
//!     }
//! }
//!
//! #[derive(Debug)]
//! enum CompilationError<'code_input>{
//!     MissingVariable {variable_name : &'code_input str},
//!     MissingFunction {function_name : &'code_input str},
//!     RootCompilationError
//! }
//! ```
//!
//! # Features
//!
//! - ``std``: Implements the Error trait for SimpleError, it might also be used for future
//! implementations that might require targeting std.
//! - ``colorization``: Allows the colorization markers functions to be used on SimpleErrorExplanation,
//! helping you to create beautiful colored error message to direct your user's attention.
//! - ``serde``: Implements Serialize and Deserialize on SimpleErrorDisplayInfo, this is useful for
//! storing logs of errors, especially for auditing.
//!
//! Currently, the ``std`` and ``colorization`` are enabled by default.

extern crate alloc;

pub use simple_error::SimpleError;
pub use simple_error_detail::SimpleErrorDetail;
pub use simple_error_display_info::SimpleErrorDisplayInfo;
pub use simple_error_explanation::SimpleErrorExplanation;

pub mod simple_error;
pub mod simple_error_detail;
pub mod simple_error_display_info;
pub mod simple_error_explanation;

pub(crate) mod formatting;