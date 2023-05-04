//! Compilation passes

pub mod abstex;
pub mod html;
pub mod latexmk;
pub mod parser;
pub mod tikz;

/// A trait defining compilation passes
/// Compilation passes should be chainable
///
/// - T is the type of data that the pass can be applied to
/// - C is the type of context that the pass can be applied with
///
/// # Example
///
/// ```rust
/// // Example of a pass that truncates a string to a given length
///
/// use crate::passes::CompilingPass;
///
/// struct TruncatePass;
///
/// // The implementation of the pass takes a `&str` as input and returns a `String`
/// // The context in this case is a `usize` which configure the final length
/// impl CompilingPass<&str, usize> for TruncatePass {
///     type Residual = String;
///     type Error = ();
///     fn apply_with(data: &str, ctx: usize) -> Result<Self::Residual, Self::Error> {
///         Ok(data[..ctx].to_string())
///     }
///     fn apply(data: &str) -> Result<Self::Residual, Self::Error> {
///         Ok(data.to_string())
///     }
/// }
///
/// assert_eq!(TruncatePass::apply_with("Hello", 3), Ok("Hel".to_string()));
/// assert_eq!("Hello".chain_pass_with::<TruncatePass, usize>(3), Ok("Hel".to_string()));
/// ```
///
pub trait CompilingPass<T, C = ()> {
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

    /// Transforms a given input of type T to a given output of type T given a
    /// context
    ///
    /// # Errors
    ///
    /// May return errors if type is not compatible
    fn apply_with(data: T, _: C) -> Result<Self::Residual, Self::Error> {
        // The default implementation provided discards any context
        Self::apply(data)
    }
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

    /// Chain current element with a given compilation pass with context data
    ///
    /// # Errors
    ///
    /// Can return a fitting error if a compilation pass cannot be applied.
    fn chain_pass_with<P: CompilingPass<Self, U>, U>(
        self,
        ctx: U,
    ) -> Result<P::Residual, P::Error> {
        P::apply_with(self, ctx)
    }
}
