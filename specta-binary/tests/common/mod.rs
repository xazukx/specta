//! Helpers shared by the integration tests.

/// Format bytes as uppercase hex, like the examples in `SPEC.md`.
pub fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parse whitespace-separated hex bytes.
pub fn from_hex(hex: &str) -> Vec<u8> {
    hex.split_whitespace()
        .map(|byte| u8::from_str_radix(byte, 16).expect("invalid hex byte"))
        .collect()
}

/// A document with no tables around a hand-written body.
pub fn doc(body: &str) -> Vec<u8> {
    from_hex(&format!("53 42 01 00 {body}"))
}

/// The bytes after the header and tables.
pub fn body(doc: &[u8]) -> &[u8] {
    let mut pos = 4;
    let varint = |pos: &mut usize| {
        let mut value = 0usize;
        for shift in (0..35).step_by(7) {
            let byte = doc[*pos];
            *pos += 1;
            value |= usize::from(byte & 0x7F) << shift;
            if byte & 0x80 == 0 {
                break;
            }
        }
        value
    };
    for flag in [1, 2] {
        if doc[3] & flag != 0 {
            for _ in 0..varint(&mut pos) {
                let len = varint(&mut pos);
                pos += len;
            }
        }
    }
    &doc[pos..]
}

/// Summarize a document as its header, the start of its body and its total size.
pub fn summary(doc: &[u8]) -> String {
    let body = body(doc);
    let head = &body[..body.len().min(12)];
    let more = if body.len() > head.len() { " …" } else { "" };
    format!(
        "{} | {}{more} ({} bytes)",
        hex(&doc[..4]),
        hex(head),
        doc.len()
    )
}
