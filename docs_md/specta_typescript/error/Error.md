# Struct `Error` -- path: `specta_typescript::error::Error`

The error type for the TypeScript exporter.

## BigInt Forbidden

Specta Typescript intentionally forbids exporting BigInt-style Rust integer types.
This includes [usize], [isize], [i64], [u64], [u128], [i128] and [f128].

This guard exists because `JSON.parse` will truncate large integers to fit into a JavaScript `number` type so we explicitly forbid exporting them.

If you encounter this error, there are a few common migration paths (in order of preference):

1. Use a smaller integer types (any of `u8`/`i8`/`u16`/`i16`/`u32`/`i32`/`f64`).
   - Only possible when the biggest integer you need to represent is small enough to be represented by a `number` in JS.
   - This approach forces your application code to handle overflow/underflow values explicitly
   - Downside is that it can introduce annoying glue code and doesn't actually work if your need large values.

2. Serialize the value as a string
    - This can be done using `#[specta(type = String)]` combined with a Serde `#[serde(with = "...")]` attribute.
    - Downside is that it can introduce annoying glue code, both on in Rust and in JS as you will need to turn it back into a `new BigInt(myString)` in JS.

3. Use a Specta-based framework
    - Frameworks like [Tauri Specta](https://github.com/specta-rs/tauri-specta) and [TauRPC](https://github.com/MatsDK/TauRPC) take care of this for you.
    - They use special internals to preserve the values and make use of [`specta-tags`](http://docs.rs/specta-tags) for generating glue-code automatically.

4. UNSAFE: Accept precision loss
    - Accept that large numbers may be deserialized differently and use `#[specta(type = f64)]` to bypass this warning on a per-field basis.
    - This can't be set globally as it is designed intentionally to introduce friction, as you are accepting the risk of data loss which is not okay.


```rust
pub struct Error {
	kind: ErrorKind,
}
```

## Methods

```rust
/**
`framework` -- Construct an error for framework-specific logic.
*/
pub fn framework<impl Into<Cow<'static, str>>, impl Into<Box<dyn std::error::Error + Send + Sync>>>(message: impl , source: impl ) -> Self
```

