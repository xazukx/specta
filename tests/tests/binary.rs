//! Cross-language conformance between `specta-binary` and its TypeScript runtime.
//!
//! Rust writes each fixture to `tests/binary-ts/fixtures` as `<name>.bin` (key table on),
//! `<name>.nokeys.bin` (key table off) and `<name>.json` (from `serde_json`). The TS tests
//! (run with `node --test`) check that `decode` matches `JSON.parse` of the JSON, that
//! re-encoding gives identical bytes, and write their own `fixtures/ts/*.bin` for Rust to decode.
//! Everything is skipped when `node` is not on `PATH`.

use std::{
    collections::BTreeMap,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;
use specta_binary::{Config, from_slice, to_vec_with};

fn project_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("binary-ts")
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct User {
    name: String,
    email: Option<String>,
    age: u8,
}

fn user(i: u32) -> User {
    User {
        name: format!("user {i}"),
        email: i.is_multiple_of(3).then(|| format!("{i}@example.com")),
        age: (i % 100) as u8,
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum External {
    Unit,
    Newtype(String),
    Tuple(u8, Option<bool>),
    Struct { a: u8, b: Vec<u8> },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Internal {
    Unit,
    Struct { a: u8, b: Option<String> },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum Adjacent {
    Unit,
    Tuple(u8, u8),
    Struct { a: String },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum Untagged {
    Int(i64),
    Text(String),
    Pair { x: f64, y: f64 },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Flattened {
    id: u32,
    #[serde(flatten)]
    kind: Internal,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Skipping {
    #[serde(skip_serializing_if = "Option::is_none")]
    maybe: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patch: Option<Option<String>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    list: Vec<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Bytes {
    #[serde(with = "serde_bytes")]
    compact: Vec<u8>,
    plain: Vec<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Tree {
    value: i32,
    children: Vec<Tree>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum Color {
    Red,
    Green,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Unit;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Empty {}

/// A fixture as the TS tests see it, listed in `fixtures/manifest.json`.
#[derive(Serialize)]
struct Fixture {
    name: &'static str,
    /// Whether `encode(decode(bin))` must reproduce the bytes exactly.
    reencode: bool,
    /// Extra `DecodeOptions` for the comparison against `JSON.parse`.
    decode: serde_json::Value,
}

struct Fixtures {
    dir: PathBuf,
    manifest: Vec<Fixture>,
}

impl Fixtures {
    fn add<T: Serialize>(&mut self, name: &'static str, value: &T) -> &mut Fixture {
        for (config, suffix) in [
            (Config::new(), "bin"),
            (Config::new().key_table(false), "nokeys.bin"),
        ] {
            let bytes = to_vec_with(value, &config).expect("serialize fixture");
            fs::write(self.dir.join(format!("{name}.{suffix}")), bytes).expect("write fixture");
        }
        let json = serde_json::to_string(value).expect("serde_json fixture");
        fs::write(self.dir.join(format!("{name}.json")), json).expect("write fixture");

        self.manifest.push(Fixture {
            name,
            reencode: true,
            decode: json!({}),
        });
        self.manifest.last_mut().expect("just pushed")
    }
}

fn write_fixtures(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(dir).expect("create fixtures dir");
    let mut fixtures = Fixtures {
        dir: dir.to_path_buf(),
        manifest: Vec::new(),
    };

    fixtures.add("user", &user(3));
    fixtures.add("users", &(0..300).map(user).collect::<Vec<_>>());
    fixtures.add(
        "strings",
        &[
            String::new(),
            "short".into(),
            "a".repeat(255),
            "b".repeat(256),
            "é🦀 ünïcödé".into(),
            "\u{FEFF}leading BOM".into(),
            "ü".repeat(200),
        ],
    );
    fixtures.add(
        "safe_integers",
        &[
            0,
            -1,
            i64::from(i32::MAX),
            i64::from(i32::MAX) + 1,
            i64::from(i32::MIN) - 1,
            -(1 << 40),
            (1 << 53) - 1,
        ],
    );
    // JSON.parse rounds these to the nearest double, so compare with `int64: "number"`.
    fixtures
        .add(
            "unsafe_integers",
            &(i64::MIN, i64::MAX, 1i64 << 53, i64::MAX as u64, -(1i64 << 60), i128::from(i64::MIN) / 2),
        )
        .decode = json!({ "int64": "number" });
    fixtures.add("floats", &[1.5, -0.25, 1e300, 5e-324, 0.1, f64::MAX]);
    // Integral floats and `-0.0` come back from JS as integers.
    fixtures.add("integral_floats", &[2.0, -0.0, 1e20]).reencode = false;
    // JS re-encodes every float as `Float64`.
    fixtures
        .add(
            "f32",
            &[0.1f32, 1.2345, 1e-7, f32::MAX, 16_777_216.0, -0.0, 0.3],
        )
        .reencode = false;
    fixtures.add(
        "external_enum",
        &[
            External::Unit,
            External::Newtype("x".into()),
            External::Tuple(1, None),
            External::Struct { a: 2, b: vec![3] },
        ],
    );
    fixtures.add(
        "internal_enum",
        &[
            Internal::Unit,
            Internal::Struct {
                a: 1,
                b: Some("b".into()),
            },
        ],
    );
    fixtures.add(
        "adjacent_enum",
        &[
            Adjacent::Unit,
            Adjacent::Tuple(1, 2),
            Adjacent::Struct { a: "a".into() },
        ],
    );
    fixtures.add(
        "untagged_enum",
        &[
            Untagged::Int(-5),
            Untagged::Text("t".into()),
            Untagged::Pair { x: 0.5, y: -1.5 },
        ],
    );
    fixtures.add(
        "flatten",
        &Flattened {
            id: 1,
            kind: Internal::Struct { a: 9, b: None },
            extra: BTreeMap::from([("z".into(), json!([1, { "y": null }]))]),
        },
    );
    fixtures.add(
        "skip_serializing_if",
        &[
            Skipping {
                maybe: None,
                patch: None,
                list: vec![],
            },
            Skipping {
                maybe: Some(1),
                patch: Some(None),
                list: vec![1],
            },
            Skipping {
                maybe: None,
                patch: Some(Some("p".into())),
                list: vec![],
            },
        ],
    );
    fixtures.add(
        "maps",
        &(
            BTreeMap::from([(1u32, "one"), (2, "two"), (10, "ten")]),
            BTreeMap::from([(false, 0), (true, 1)]),
            BTreeMap::from([(Color::Red, 'r'), (Color::Green, 'g')]),
        ),
    );
    // JS objects put integer-like keys first in numeric order, so the bytes differ.
    fixtures
        .add(
            "integer_like_string_keys",
            &BTreeMap::from([("1", 1), ("10", 10), ("2", 2), ("a", 0)]),
        )
        .reencode = false;
    fixtures.add(
        "proto_keys",
        &BTreeMap::from([
            ("__proto__", json!({ "polluted": true })),
            ("constructor", json!("c")),
            ("toString", json!(1)),
        ]),
    );
    fixtures.add(
        "bytes",
        &Bytes {
            compact: (0..300).map(|i| i as u8).collect(),
            plain: vec![0, 1, 255],
        },
    );
    fixtures.add(
        "recursive",
        &Tree {
            value: 1,
            children: vec![
                Tree {
                    value: 2,
                    children: vec![],
                },
                Tree {
                    value: 3,
                    children: vec![Tree {
                        value: 4,
                        children: vec![],
                    }],
                },
            ],
        },
    );
    fixtures.add(
        "json_value",
        &json!({ "a": [1, "two", null, true, { "b": 2.5 }], "c": {}, "d": [] }),
    );
    fixtures.add(
        "scalars",
        &('x', '🦀', (), Unit, None::<u8>, Some(5u8), i128::from(i32::MIN) * 1000, false),
    );
    fixtures.add(
        "empty",
        &(Vec::<i32>::new(), BTreeMap::<String, i32>::new(), Empty {}),
    );
    fixtures.add(
        "large_object",
        &(0..300)
            .map(|i| (format!("key{i:03}"), i))
            .collect::<BTreeMap<_, _>>(),
    );
    let mut deep = json!(1);
    for _ in 0..100 {
        deep = json!([deep]);
    }
    fixtures.add("deep", &deep);

    let manifest = serde_json::to_string_pretty(&fixtures.manifest).expect("manifest");
    fs::write(dir.join("manifest.json"), manifest).expect("write manifest");
}

/// How to run a `.ts` test file with the `node` on `PATH`, or `None` if it can't.
fn node_test_command() -> Option<Command> {
    let output = Command::new("node").arg("--version").output().ok()?;
    let version = String::from_utf8_lossy(&output.stdout);
    let mut parts = version.trim().trim_start_matches('v').split('.');
    let major: u32 = parts.next()?.parse().ok()?;
    let minor: u32 = parts.next()?.parse().ok()?;

    // Type stripping is on by default from Node 22.18 and 23.6, and behind a flag from 22.6.
    let mut command = Command::new("node");
    match (major, minor) {
        (23.., _) if (major, minor) >= (23, 6) => {}
        (22, 18..) => {}
        (22, 6..) | (23, _) => {
            command.arg("--experimental-strip-types");
        }
        _ => {
            let tsx = if cfg!(windows) { "tsx.cmd" } else { "tsx" };
            Command::new(tsx).arg("--version").output().ok()?;
            command = Command::new(tsx);
        }
    }
    command.arg("--test");
    Some(command)
}

/// Write the Rust fixtures and run the TS tests once per test run.
/// `None` means `node` is unavailable and the cross-language tests are skipped.
fn conformance_run() -> Option<&'static Result<(), String>> {
    static RUN: OnceLock<Option<Result<(), String>>> = OnceLock::new();
    RUN.get_or_init(|| {
        let dir = project_dir();
        write_fixtures(&dir.join("fixtures"));

        let Some(mut command) = node_test_command() else {
            eprintln!("skipping specta-binary TypeScript conformance tests: `node` with TypeScript support is not on PATH");
            return None;
        };
        let output = command
            .args(["conformance.test.ts", "hostile.test.ts"])
            .current_dir(&dir)
            .output()
            .expect("run node --test");
        Some(match output.status.success() {
            true => Ok(()),
            false => Err(format!(
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )),
        })
    })
    .as_ref()
}

/// A test failure whose `Debug` output is the raw message, so node's output stays readable.
struct Failure(String);

impl Debug for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl<T: std::fmt::Display> From<T> for Failure {
    fn from(err: T) -> Self {
        Self(err.to_string())
    }
}

#[test]
fn binary_typescript_conformance() -> Result<(), Failure> {
    match conformance_run() {
        Some(Err(output)) => Err(Failure(format!(
            "TypeScript conformance tests failed:\n{output}"
        ))),
        _ => Ok(()),
    }
}

/// Decode `fixtures/ts/<name>.bin` and `<name>.nokeys.bin`, written by the TS encoder.
fn decode_ts<T: DeserializeOwned>(name: &str) -> Result<[T; 2], Failure> {
    let dir = project_dir().join("fixtures").join("ts");
    let decode = |suffix: &str| -> Result<T, Failure> {
        let path = dir.join(format!("{name}.{suffix}"));
        let bytes = fs::read(&path).map_err(|err| format!("read {}: {err}", path.display()))?;
        Ok(from_slice(&bytes).map_err(|err| format!("decode {}: {err}", path.display()))?)
    };
    Ok([decode("bin")?, decode("nokeys.bin")?])
}

fn check_ts<T: DeserializeOwned + PartialEq + Debug>(name: &str, expected: T) -> Result<(), Failure> {
    for value in decode_ts::<T>(name)? {
        assert_eq!(value, expected, "fixtures/ts/{name}");
    }
    Ok(())
}

#[test]
fn binary_rust_decodes_typescript_output() -> Result<(), Failure> {
    // Only a passing run writes the TS fixtures. A failed run is reported by the test above.
    let Some(Ok(())) = conformance_run() else {
        return Ok(());
    };

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Numbers {
        small: u8,
        negative: i32,
        big: u64,
        bigint: i64,
        float: f64,
        int_as_float: f64,
        float_as_f32: f32,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Long {
        text: String,
        list: Vec<u8>,
        map: BTreeMap<String, u16>,
    }

    check_ts(
        "user",
        User {
            name: "Ann".into(),
            email: None,
            age: 30,
        },
    )?;
    check_ts(
        "user_without_email",
        User {
            name: "Bob".into(),
            email: None,
            age: 41,
        },
    )?;
    check_ts(
        "users",
        (0..300)
            .map(|i| User {
                name: format!("user {i}"),
                email: Some(format!("{i}@example.com")),
                age: (i % 100) as u8,
            })
            .collect::<Vec<_>>(),
    )?;
    let bytes = || Bytes {
        compact: vec![1, 2, 3],
        plain: vec![4, 5, 6],
    };
    check_ts("bytes", bytes())?;
    // A `number[]` feeds a `serde_bytes` field and a `Uint8Array` feeds a `Vec<u8>`.
    check_ts("bytes_swapped", bytes())?;
    check_ts(
        "numbers",
        Numbers {
            small: 1,
            negative: -5,
            big: 1 << 40,
            bigint: 1 << 60,
            float: 1.5,
            int_as_float: 2.0,
            float_as_f32: 0.1,
        },
    )?;
    for [nan, infinity, neg_infinity] in decode_ts::<[f64; 3]>("special_floats")? {
        assert!(nan.is_nan());
        assert_eq!((infinity, neg_infinity), (f64::INFINITY, f64::NEG_INFINITY));
    }
    check_ts(
        "external_enum",
        vec![
            External::Unit,
            External::Newtype("x".into()),
            External::Tuple(1, None),
            External::Struct { a: 2, b: vec![] },
        ],
    )?;
    check_ts(
        "internal_enum",
        Internal::Struct {
            a: 1,
            b: Some("x".into()),
        },
    )?;
    check_ts(
        "integer_keys",
        BTreeMap::from([(1u32, "a".to_string()), (20, "b".into())]),
    )?;
    check_ts(
        "long",
        Long {
            text: "x".repeat(300),
            list: vec![1; 300],
            map: (0..300).map(|i| (format!("k{i}"), i)).collect(),
        },
    )?;
    check_ts("unicode", "héllo 🦀 \u{FEFF}".to_string())?;
    check_ts("date", "1970-01-01T00:00:00.000Z".to_string())?;
    check_ts(
        "json_value",
        json!({ "a": [1, "two", null, true, { "b": 2.5 }], "big": 1u64 << 60, "nested": { "c": [] } }),
    )?;
    Ok(())
}
