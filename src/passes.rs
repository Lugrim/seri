//! Compilation passes

pub mod latex;
pub mod parser;

/// A trait defining compilation passes
/// Compilation passes should be chainable
pub trait CompilingPass<T, U, UE> {
    /// Transforms a given input of type T to a given output of type T
    ///
    /// # Errors
    ///
    /// May return errors if type is not compatible
    fn apply(_: T) -> Result<U, UE>;
}

/// A trait to be able to chain compilation passes
pub trait PassInput : Sized {
    /// Chain current element with a given compilation pass
    ///
    /// # Errors
    ///
    /// Can return a fitting error if a compilation pass cannot be applied.
    fn chain_pass<P, U, E>(self) -> Result<U, E> where P: CompilingPass<Self, U, E> {
        P::apply(self)
    }
}
