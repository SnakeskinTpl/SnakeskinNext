use std::any::Any;
use std::fmt::Debug;

/// Type-erased parser context stored on the input's state stack.
pub trait ParserState: Any + Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    /// Changes to the clone must not affect the original state, or vice versa.
    fn clone_box(&self) -> Box<dyn ParserState>;
}

impl Clone for Box<dyn ParserState> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
