mod tag;
pub use tag::{tag, succeed, Tag, TagError};
mod take;
pub use take::{take, Take, TakeError};
mod ltrim;
pub use ltrim::{ltrim, LTrim};

mod seq;
pub use seq::{seq, Seq};
mod or;
pub use or::{or, Or, Either, AlternativeError};
mod choice;
pub use choice::{choice, Choice};

mod repeat;
pub use repeat::{repeat, Repeat, RepeatError, RepeatErrorKind};
mod opt;
pub use opt::{opt, Opt};
mod lookup;
pub use lookup::{lookup, Lookup};
mod not;
pub use not::{not, Not, NotError};

mod rec;
pub use rec::{rec, Rec};
mod flat_map;
pub use flat_map::{flat_map, FlatMap};
mod map;
pub use map::{map, Map};

mod err;
pub use err::DataError;

pub mod input;
pub use input::{ParserInput, ParserState};

use std::error::Error;

#[cfg(test)]
mod tests;

/// Source context for a parser failure; all offsets are measured in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorLocation {
    pub start: usize,
    pub byte_position: usize,
    /// Character at the failure position, or None at end of input.
    pub actual: Option<char>,
}

pub trait ParseError: Error {
    fn location(&self) -> &ErrorLocation;

    /// Whether combinators may treat this failure as an ordinary mismatch.
    fn is_recoverable(&self) -> bool {
        true
    }

    /// Byte offset in the source where parsing failed.
    fn byte_position(&self) -> usize {
        self.location().byte_position
    }
}

/// Parsed output and remaining input, or a diagnostic on failure.
pub type ParserResult<'a, Output> = Result<(Output, ParserInput<'a>), Box<dyn ParseError>>;

/// Parses source text whose lifetime may also be used by the output.
pub trait Parser<'a> {
    type Output;

    fn parse(&self, i: ParserInput<'a>) -> ParserResult<'a, Self::Output>;
}

impl<'a, P: Parser<'a> + ?Sized> Parser<'a> for Box<P> {
    type Output = P::Output;

    fn parse(&self, i: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        (**self).parse(i)
    }
}
