//! Exports TypeScript types with the unchanged `specta-typescript` exporter, plus the
//! `specta-binary` runtime and sample documents for `main.ts` to decode.
//!
//! ```sh
//! cargo run -p example-binary-ts
//! node examples/binary-ts/main.ts
//! ```

use std::{error::Error, fs, path::Path};

use serde::{Deserialize, Serialize};
use specta::{Type, Types};
use specta_binary::Config;
use specta_typescript::Typescript;

#[derive(Debug, PartialEq, Type, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[specta(optional)]
    pub email: Option<String>,
    pub role: Role,
    pub follower_count: u32,
}

#[derive(Debug, PartialEq, Type, Serialize, Deserialize)]
pub enum Role {
    Admin,
    Member,
}

#[derive(Debug, PartialEq, Type, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    UserCreated { user: User },
    UserDeleted { id: u32 },
    Ping,
}

#[derive(Debug, PartialEq, Type, Serialize, Deserialize)]
pub struct Attachment {
    pub name: String,
    // `serde_bytes` gives the compact `ByteArray` form. `specta-typescript` has no
    // `Uint8Array` type yet, so it is typed as `number[]` and decoded with `{ bytes: "array" }`.
    #[serde(with = "serde_bytes")]
    #[specta(type = Vec<u8>)]
    pub data: Vec<u8>,
}

fn user(id: u32) -> User {
    User {
        id,
        name: format!("User {id}"),
        email: (id % 2 == 1).then(|| format!("user{id}@example.com")),
        role: if id == 1 { Role::Admin } else { Role::Member },
        follower_count: id * 1_000,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("out");
    fs::create_dir_all(&out)?;

    let types = Types::default()
        .register::<User>()
        .register::<Event>()
        .register::<Attachment>();
    Typescript::default().export_to(out.join("bindings.ts"), &specta_serde::apply(types)?)?;
    fs::write(out.join("specta_binary.ts"), specta_binary::TYPESCRIPT_RUNTIME)?;

    let ann = user(1);
    let bytes = specta_binary::to_vec(&ann)?;
    assert_eq!(specta_binary::from_slice::<User>(&bytes)?, ann);
    fs::write(out.join("user.bin"), &bytes)?;

    let events = vec![
        Event::UserCreated { user: user(2) },
        Event::UserDeleted { id: 1 },
        Event::Ping,
    ];
    fs::write(out.join("events.bin"), specta_binary::to_vec(&events)?)?;

    let attachment = Attachment {
        name: "pixel.bin".into(),
        data: vec![0xDE, 0xAD, 0xBE, 0xEF],
    };
    fs::write(out.join("attachment.bin"), specta_binary::to_vec(&attachment)?)?;

    let users = (1..=100).map(user).collect::<Vec<_>>();
    println!(
        "100 users: {} bytes as JSON, {} bytes as Specta Binary ({} without the key table)",
        serde_json::to_vec(&users)?.len(),
        specta_binary::to_vec(&users)?.len(),
        specta_binary::to_vec_with(&users, &Config::new().key_table(false))?.len(),
    );
    println!("Wrote bindings, runtime and samples to {}", out.display());
    Ok(())
}
