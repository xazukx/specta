/// Output format for the constant value in TypeScript.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstantOutput {
    /// Infer the output format from the Rust type (default).
    Infer,
    /// Force output as a string literal.
    String,
    /// Force output as a byte array.
    Bytes,
}

impl Default for ConstantOutput {
    fn default() -> Self {
        Self::Infer
    }
}
