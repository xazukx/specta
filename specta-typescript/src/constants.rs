use std::fmt::Write as _;

use specta::{Constants, datatype::ConstantValue};

use crate::{Error, Exporter, legacy::escape_typescript_string_literal};

/// Render a single constant as a TypeScript `export const` declaration.
pub fn export_constant(
    exporter: &Exporter,
    constant: &specta::NamedConstant,
) -> Result<String, Error> {
    let mut s = String::new();
    export_constant_internal(&mut s, exporter, constant)?;
    Ok(s)
}

pub(crate) fn export_constant_internal(
    s: &mut String,
    _exporter: &Exporter,
    constant: &specta::NamedConstant,
) -> Result<(), Error> {
    // JSDoc comment for docs/deprecation
    let has_docs = !constant.docs.is_empty();
    let has_deprecated = constant.deprecated.is_some();

    if has_docs || has_deprecated {
        s.push_str("/**\n");

        if has_docs {
            for line in constant.docs.lines() {
                write!(s, " * {}\n", line).unwrap();
            }
        }

        if let Some(deprecated) = &constant.deprecated {
            s.push_str(" * @deprecated");
            if let Some(note) = deprecated.note() {
                write!(s, " {}", note).unwrap();
            }
            s.push('\n');
        }

        s.push_str(" */\n");
    }

    s.push_str("export const ");
    s.push_str(&constant.name);
    s.push_str(" = ");

    render_constant_value(s, &constant.value);

    s.push_str(" as const;\n");

    Ok(())
}

/// Render a collection of constants as TypeScript declarations.
pub fn export_constants(exporter: &Exporter, constants: &Constants) -> Result<String, Error> {
    let mut s = String::new();
    export_constants_internal(&mut s, exporter, constants)?;
    Ok(s)
}

pub(crate) fn export_constants_internal(
    s: &mut String,
    exporter: &Exporter,
    constants: &Constants,
) -> Result<(), Error> {
    for constant in constants.into_sorted_iter() {
        if !s.is_empty() && !s.ends_with('\n') {
            s.push('\n');
        }
        export_constant_internal(s, exporter, constant)?;
    }
    Ok(())
}

fn render_constant_value(s: &mut String, value: &ConstantValue) {
    match value {
        ConstantValue::String(v) => {
            write!(s, "\"{}\"", escape_typescript_string_literal(v)).unwrap();
        }
        ConstantValue::Integer(n) => {
            write!(s, "{}", n).unwrap();
        }
        ConstantValue::UnsignedInteger(n) => {
            write!(s, "{}", n).unwrap();
        }
        ConstantValue::Float(bits) => {
            let v = bits.to_f64();
            if v.is_nan() {
                s.push_str("NaN");
            } else if v.is_infinite() {
                if v.is_sign_positive() {
                    s.push_str("Infinity");
                } else {
                    s.push_str("-Infinity");
                }
            } else {
                // Format with enough precision
                let formatted = format!("{}", v);
                s.push_str(&formatted);
            }
        }
        ConstantValue::Bool(b) => {
            s.push_str(if *b { "true" } else { "false" });
        }
        ConstantValue::Bytes(bytes) => {
            s.push_str("[");
            for (i, byte) in bytes.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                write!(s, "{}", byte).unwrap();
            }
            s.push_str("]");
        }
        ConstantValue::Null => {
            s.push_str("null");
        }
    }
}
