# Trait `Type` -- path: `specta::type::Type`

Provides runtime type information that can be fed into a language exporter to generate a type definition for another language.
Avoid implementing this trait yourself where possible and use the [`Type`] macro instead.

This should be only implemented by the [`Type`] macro.
TODO: Discuss how to avoid custom implementations.

- auto: no | unsafe: no | dyn-compatible: no


## Methods

```rust
/**
`definition` -- returns a [`DataType`] that represents the type.
This will also register this and any dependent types into the [`Types`].
*/
fn definition(types: &mut Types) -> DataType
```

## Implemented by

- [T](./generics/T.md)
- [K](./generics/K.md)
- [V](./generics/V.md)
- [E](./generics/E.md)
- [L](./generics/L.md)
- [R](./generics/R.md)
- [PrimitiveSet<T>](./impls/PrimitiveSet.md)
- [PrimitiveMap<K, V>](./impls/PrimitiveMap.md)
- i8
- i16
- i32
- i64
- i128
- isize
- u8
- u16
- u32
- u64
- u128
- usize
- f32
- f64
- bool
- char
- str
- f16
- f128
- (T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13)
- (T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13)
- (T4, T5, T6, T7, T8, T9, T10, T11, T12, T13)
- (T5, T6, T7, T8, T9, T10, T11, T12, T13)
- (T6, T7, T8, T9, T10, T11, T12, T13)
- (T7, T8, T9, T10, T11, T12, T13)
- (T8, T9, T10, T11, T12, T13)
- (T9, T10, T11, T12, T13)
- (T10, T11, T12, T13)
- (T11, T12, T13)
- (T12, T13)
- (T13)
- ()
- std::string::String
- std::vec::Vec<T>
- std::collections::VecDeque<T>
- std::collections::BinaryHeap<T>
- std::collections::LinkedList<T>
- std::collections::HashSet<T>
- std::collections::BTreeSet<T>
- std::collections::HashMap<K, V>
- std::collections::BTreeMap<K, V>
- std::boxed::Box<T>
- std::rc::Rc<T>
- std::sync::Arc<T>
- std::cell::Cell<T>
- std::cell::RefCell<T>
- std::sync::Mutex<T>
- std::sync::RwLock<T>
- std::ffi::CString
- std::ffi::CStr
- std::ffi::OsString
- std::ffi::OsStr
- std::path::Path
- std::path::PathBuf
- std::net::IpAddr
- std::net::Ipv4Addr
- std::net::Ipv6Addr
- std::net::SocketAddr
- std::net::SocketAddrV4
- std::net::SocketAddrV6
- std::sync::atomic::AtomicBool
- std::sync::atomic::AtomicI8
- std::sync::atomic::AtomicI16
- std::sync::atomic::AtomicI32
- std::sync::atomic::AtomicIsize
- std::sync::atomic::AtomicU8
- std::sync::atomic::AtomicU16
- std::sync::atomic::AtomicU32
- std::sync::atomic::AtomicUsize
- std::sync::atomic::AtomicI64
- std::sync::atomic::AtomicU64
- std::num::NonZeroU8
- std::num::NonZeroU16
- std::num::NonZeroU32
- std::num::NonZeroU64
- std::num::NonZeroUsize
- std::num::NonZeroI8
- std::num::NonZeroI16
- std::num::NonZeroI32
- std::num::NonZeroI64
- std::num::NonZeroIsize
- std::num::NonZeroU128
- std::num::NonZeroI128
- std::ops::Range<T>
- std::ops::RangeInclusive<T>
- std::convert::Infallible
- std::time::SystemTime
- std::time::Duration
- std::borrow::Cow<''a, T>
- &T
- [T]
- [T; N]
- Option<T>
- std::marker::PhantomData<T>
- std::result::Result<T, E>

