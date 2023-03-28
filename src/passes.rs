//! Compilation passes

pub mod latex;
pub mod parser;

/// A trait defining compilation passes
/// Compilation passes should be chainable
pub trait CompilingPass<T> {
    /// Type of data that is returned by the pass if it succeeds.
    type Residual;
    /// Type of errors that the pass returns when it fails.
    type Error;

    /// Transforms a given input of type T to a given output of type T
    ///
    /// # Errors
    ///
    /// May return errors if type is not compatible
    fn apply(_: T) -> Result<Self::Residual, Self::Error>;
}

/// A trait to be able to chain compilation passes
pub trait PassInput: Sized {
    /// Chain current element with a given compilation pass
    ///
    /// # Errors
    ///
    /// Can return a fitting error if a compilation pass cannot be applied.
    fn chain_pass<P>(self) -> Result<P::Residual, P::Error>
    where
        P: CompilingPass<Self>,
    {
        P::apply(self)
    }
}
