use crate::{Error, JsonSchema};
use serde_json::{Map, Value, json};
use specta::{
    Types,
    datatype::{NamedDataType, *},
};

/// Convert a NamedDataType to a JSON Schema definition
pub fn export(js: &JsonSchema, types: &Types, ndt: &NamedDataType) -> Result<Value, Error> {
    let mut schema = datatype_to_schema(js, types, ndt.ty(), true)?;

    // Add title from type name and description from docs
    if let Some(obj) = schema.as_object_mut() {
        obj.insert("title".to_string(), Value::String(ndt.name().to_string()));

        let docs = ndt.docs();
        if !docs.is_empty() {
            obj.insert("description".to_string(), Value::String(docs.to_string()));
        }
    }

    Ok(schema)
}

/// Convert a DataType to a JSON Schema, optionally as a reference
pub fn datatype_to_schema(
    js: &JsonSchema,
    types: &Types,
    dt: &DataType,
    is_definition: bool,
) -> Result<Value, Error> {
    match dt {
        DataType::Primitive(p) => Ok(primitive_to_schema(p)),

        DataType::Nullable(inner) => {
            let inner_schema = datatype_to_schema(js, types, inner, false)?;
            Ok(json!({
                "anyOf": [
                    inner_schema,
                    {"type": "null"}
                ]
            }))
        }

        DataType::List(list) => {
            let items = datatype_to_schema(js, types, list.ty(), false)?;

            if let Some(len) = list.length() {
                Ok(json!({
                    "type": "array",
                    "items": items,
                    "minItems": len,
                    "maxItems": len
                }))
            } else {
                Ok(json!({
                    "type": "array",
                    "items": items
                }))
            }
        }

        DataType::Map(map) => {
            let value_schema = datatype_to_schema(js, types, map.value_ty(), false)?;
            Ok(json!({
                "type": "object",
                "additionalProperties": value_schema
            }))
        }

        DataType::Struct(s) => struct_to_schema(js, types, s),

        DataType::Enum(e) => enum_to_schema(js, types, e),

        DataType::Tuple(t) => tuple_to_schema(js, types, t),

        DataType::Reference(r) => match r {
            Reference::Named(r) => {
                let referenced_ndt = r.get(types).ok_or_else(|| {
                    Error::InvalidReference("Reference not found in Types".to_string())
                })?;

                // Inline types (e.g. String, Vec<T>) are always resolved directly,
                // never emitted as $ref. This matches the TypeScript exporter's
                // `r.inline()` check.
                if is_definition || r.inline() {
                    datatype_to_schema(js, types, referenced_ndt.ty(), is_definition)
                } else {
                    let defs_key = js.schema_version.definitions_key();
                    Ok(json!({
                        "$ref": format!("#/{}/{}", defs_key, referenced_ndt.name())
                    }))
                }
            }
            Reference::Opaque(_) => Err(Error::UnsupportedDataType(
                "Opaque references are not supported by JSON Schema exporter".to_string(),
            )),
            Reference::Generic(_) => Ok(json!({})),
        },
    }
}

fn primitive_to_schema(p: &Primitive) -> Value {
    match p {
        Primitive::bool => json!({"type": "boolean"}),
        Primitive::str => json!({"type": "string"}),
        Primitive::char => json!({"type": "string", "minLength": 1, "maxLength": 1}),

        Primitive::i8 => json!({"type": "integer", "minimum": i8::MIN, "maximum": i8::MAX}),
        Primitive::i16 => json!({"type": "integer", "minimum": i16::MIN, "maximum": i16::MAX}),
        Primitive::i32 => json!({"type": "integer", "format": "int32"}),
        Primitive::i64 => json!({"type": "integer", "format": "int64"}),
        Primitive::i128 => json!({"type": "integer"}),
        Primitive::isize => json!({"type": "integer"}),

        Primitive::u8 => json!({"type": "integer", "minimum": 0, "maximum": u8::MAX}),
        Primitive::u16 => json!({"type": "integer", "minimum": 0, "maximum": u16::MAX}),
        Primitive::u32 => json!({"type": "integer", "minimum": 0, "format": "uint32"}),
        Primitive::u64 => json!({"type": "integer", "minimum": 0, "format": "uint64"}),
        Primitive::u128 => json!({"type": "integer", "minimum": 0}),
        Primitive::usize => json!({"type": "integer", "minimum": 0}),

        Primitive::f16 => json!({"type": "number", "format": "float16"}),
        Primitive::f32 => json!({"type": "number", "format": "float"}),
        Primitive::f64 => json!({"type": "number", "format": "double"}),
        Primitive::f128 => json!({"type": "number", "format": "float128"}),
    }
}

/// Build a JSON Schema object from named fields, supporting flattened fields via allOf.
fn named_fields_to_schema(
    js: &JsonSchema,
    types: &Types,
    fields: &NamedFields,
) -> Result<Value, Error> {
    let mut properties = Map::new();
    let mut required = Vec::new();
    let mut all_of_parts = Vec::new();

    for (name, (field, ty)) in fields
        .fields()
        .iter()
        .filter_map(|(name, field)| field.ty().map(|ty| (name, (field, ty))))
    {
        if field.flatten() {
            all_of_parts.push(datatype_to_schema(js, types, ty, false)?);
        } else {
            let schema = datatype_to_schema(js, types, ty, false)?;
            properties.insert(name.clone().into_owned(), schema);
            if !field.optional() {
                required.push(Value::String(name.clone().into_owned()));
            }
        }
    }

    let mut obj = json!({
        "type": "object",
        "properties": properties
    });

    if !required.is_empty() {
        obj.as_object_mut()
            .unwrap()
            .insert("required".to_string(), Value::Array(required));
    }

    if all_of_parts.is_empty() {
        Ok(obj)
    } else {
        all_of_parts.insert(0, obj);
        Ok(json!({"allOf": all_of_parts}))
    }
}

fn struct_to_schema(js: &JsonSchema, types: &Types, s: &Struct) -> Result<Value, Error> {
    match s.fields() {
        Fields::Unit => Ok(json!({"type": "null"})),
        Fields::Unnamed(fields) => {
            let items: Result<Vec<_>, _> = fields
                .fields()
                .iter()
                .filter_map(|field| field.ty().map(|ty| (field, ty)))
                .map(|(_, ty)| datatype_to_schema(js, types, ty, false))
                .collect();

            let items = items?;
            Ok(json!({
                "type": "array",
                "prefixItems": items,
                "items": false,
                "minItems": items.len(),
                "maxItems": items.len()
            }))
        }
        Fields::Named(fields) => named_fields_to_schema(js, types, fields),
    }
}

fn enum_to_schema(js: &JsonSchema, types: &Types, e: &Enum) -> Result<Value, Error> {
    let filtered: Vec<_> = e
        .variants()
        .iter()
        .filter(|(_, variant)| !variant.skip())
        .collect();

    if filtered.is_empty() {
        return Err(Error::ConversionError(
            "Enum has no non-skipped variants".to_string(),
        ));
    }

    // String-only enums: use compact form
    if filtered
        .iter()
        .all(|(_, v)| matches!(v.fields(), Fields::Unit))
    {
        if filtered.len() == 1 {
            // Single literal: {"const": "X"}
            return Ok(json!({"const": filtered[0].0.as_ref()}));
        }
        // Multiple literals: {"enum": ["A", "B", "C"]}
        let names: Vec<Value> = filtered
            .iter()
            .map(|(name, _)| Value::String(name.to_string()))
            .collect();
        return Ok(json!({"enum": names}));
    }

    let variants: Result<Vec<_>, _> = filtered
        .iter()
        .map(|(name, variant)| variant_to_schema(js, types, name, variant))
        .collect();
    let variants = variants?;

    if variants.len() == 1 {
        Ok(variants.into_iter().next().unwrap())
    } else {
        Ok(json!({"anyOf": variants}))
    }
}

/// Convert a single enum variant to JSON Schema.
///
/// After `specta_serde::apply()`, variant fields already contain the correct
/// tagging structure (external, internal, adjacent, or untagged). This function
/// renders variant fields generically, matching the TypeScript/Zod exporter pattern.
fn variant_to_schema(
    js: &JsonSchema,
    types: &Types,
    name: &str,
    variant: &Variant,
) -> Result<Value, Error> {
    match variant.fields() {
        Fields::Unit => {
            // Unit variant = string literal constant
            Ok(json!({"const": name}))
        }
        Fields::Unnamed(fields) => {
            // After serde transformation, unnamed fields contain the variant's
            // wire representation. A single field is unwrapped directly.
            let items: Result<Vec<_>, _> = fields
                .fields()
                .iter()
                .filter_map(|field| field.ty().map(|ty| (field, ty)))
                .map(|(_, ty)| datatype_to_schema(js, types, ty, false))
                .collect();

            let items = items?;

            if items.len() == 1 {
                Ok(items.into_iter().next().unwrap())
            } else {
                Ok(json!({
                    "type": "array",
                    "prefixItems": items,
                    "items": false,
                    "minItems": items.len(),
                    "maxItems": items.len()
                }))
            }
        }
        Fields::Named(fields) => {
            // After serde transformation, named fields contain the full object
            // structure including any tag/content fields from the tagging mode.
            named_fields_to_schema(js, types, fields)
        }
    }
}

fn tuple_to_schema(js: &JsonSchema, types: &Types, t: &Tuple) -> Result<Value, Error> {
    if t.elements().is_empty() {
        return Ok(json!({"type": "null"}));
    }

    let items: Result<Vec<_>, _> = t
        .elements()
        .iter()
        .map(|ty| datatype_to_schema(js, types, ty, false))
        .collect();

    let items = items?;
    Ok(json!({
        "type": "array",
        "prefixItems": items,
        "items": false,
        "minItems": items.len(),
        "maxItems": items.len()
    }))
}
