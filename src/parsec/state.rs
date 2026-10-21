use super::*;

#[cfg(test)]
mod tests;

/// Creates fresh local state for each parse attempt.
/// Backtracking within the callback does not restore this state.
pub fn state<'a, S, Init, F, O>(init: Init, parse: F) -> State<Init, F>
where
    Init: Fn() -> S,
    F: Fn(&mut S, ParserInput<'a>) -> ParserResult<'a, O>,
{
    State(init, parse)
}

#[derive(Debug)]
pub struct State<Init, F>(Init, F);

impl<'a, S, Init, F, O> Parser<'a> for State<Init, F>
where
    Init: Fn() -> S,
    F: Fn(&mut S, ParserInput<'a>) -> ParserResult<'a, O>,
{
    type Output = O;

    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Self::Output> {
        let mut state = self.0();
        self.1(&mut state, input)
    }
}
