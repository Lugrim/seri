//! Compilation passes

pub mod latex;
pub mod parser;

/// A trait definint compilation passes
/// Compilation passes should be chainable
pub trait CompilingPass<T, U, UE> {
    /// Transforms a given input of type T to a given output of type T
    ///
    /// # Errors
    ///
    /// May return errors if type is not compatible
    fn apply(_: T) -> Result<U, UE>;
}
