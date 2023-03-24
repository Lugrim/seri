pub mod parser;

pub trait CompilingPass<T, U, UE> {
    fn apply(_: T) -> Result<U, UE>;
}
