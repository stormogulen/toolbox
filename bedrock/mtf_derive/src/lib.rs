
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{Data, DeriveInput, Expr, ExprLit, Fields, Lit, Type, TypePath, parse_macro_input};

/// Get the size in bytes for primitive types.
fn primitive_size_bytes(ident: &str) -> Option<usize> {
    match ident {
        "u8" | "i8" | "bool" => Some(1),
        "u16" | "i16" => Some(2),
        "u32" | "i32" | "f32" => Some(4),
        "u64" | "i64" | "f64" => Some(8),
        "u128" | "i128" => Some(16),
        _ => None,
    }
}

/// Calculate the size of a type in bytes.
fn type_size_and_check(ty: &Type) -> Result<usize, String> {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            let ident = path.segments.last().unwrap().ident.to_string();
            primitive_size_bytes(&ident).ok_or_else(|| format!("unsupported type `{}`", ident))
        }
        Type::Array(arr) => {
            let elem_size = type_size_and_check(&arr.elem)?;
            if let Expr::Lit(ExprLit {
                lit: Lit::Int(len), ..
            }) = &arr.len
            {
                let count = len.base10_parse::<usize>().map_err(|e| e.to_string())?;
                Ok(elem_size * count)
            } else {
                Err("array length must be a literal integer".to_string())
            }
        }
        _ => Err(format!("unsupported type: {}", quote! { #ty })),
    }
}

/// Check if the type has #[repr(C)] or #[repr(C, packed)]
fn check_repr_c(input: &DeriveInput) -> bool {
    input.attrs.iter().any(|attr| {
        if attr.path().is_ident("repr") {
            attr.to_token_stream().to_string().contains('C')
        } else {
            false
        }
    })
}

#[proc_macro_derive(MTF)]
pub fn derive_mtf(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident.to_string();

    if !check_repr_c(&input) {
        return syn::Error::new_spanned(
            &input.ident,
            "MTF derive requires #[repr(C)] or #[repr(C, packed)]",
        )
        .to_compile_error()
        .into();
    }

    let mut fields_info = Vec::<(String, usize)>::new();
    let mut total_size = 0usize;

    if let Data::Struct(ds) = &input.data {
        if let Fields::Named(named) = &ds.fields {
            for f in named.named.iter() {
                let fname = f.ident.as_ref().unwrap().to_string();
                match type_size_and_check(&f.ty) {
                    Ok(sz) => {
                        total_size += sz;
                        fields_info.push((fname, sz));
                    }
                    Err(e) => return syn::Error::new_spanned(&f.ty, e).to_compile_error().into(),
                }
            }
        } else {
            return syn::Error::new_spanned(&input.ident, "Only named fields supported")
                .to_compile_error()
                .into();
        }
    } else {
        return syn::Error::new_spanned(&input.ident, "Only structs supported")
            .to_compile_error()
            .into();
    }

    // Build string table
    let mut strings = Vec::new();
    let type_name_offset = 0u32;

    strings.extend_from_slice(name.as_bytes());
    strings.push(0);

    let mut field_name_offsets = Vec::new();
    for (fname, _) in &fields_info {
        let offset = strings.len() as u32;
        field_name_offsets.push(offset);
        strings.extend_from_slice(fname.as_bytes());
        strings.push(0);
    }

    // Build MTF blob
    let mut blob = Vec::new();
    blob.extend_from_slice(b"MTF\0");
    blob.extend_from_slice(&1u32.to_le_bytes());
    blob.extend_from_slice(&1u32.to_le_bytes());
    blob.extend_from_slice(&type_name_offset.to_le_bytes());
    blob.extend_from_slice(&((total_size * 8) as u32).to_le_bytes());
    blob.extend_from_slice(&(fields_info.len() as u32).to_le_bytes());

    let mut offset_bits = 0usize;
    for (i, (_fname, sz)) in fields_info.iter().enumerate() {
        let name_off = field_name_offsets[i];
        blob.extend_from_slice(&name_off.to_le_bytes());
        blob.extend_from_slice(&(offset_bits as u32).to_le_bytes());
        blob.extend_from_slice(&((sz * 8) as u32).to_le_bytes());
        offset_bits += sz * 8;
    }

    blob.extend_from_slice(&(strings.len() as u32).to_le_bytes());
    blob.extend_from_slice(&strings);

    let blob_bytes = blob.iter().map(|b| quote! { #b }).collect::<Vec<_>>();

    let ident = &input.ident;

    let expanded = quote! {
        impl mtf::MTFType for #ident {
            fn mtf_type_blob() -> &'static [u8] {
                &[ #( #blob_bytes ),* ]
            }

            fn mtf_string_table() -> &'static [u8] {
                &[]
            }
        }
    };

    expanded.into()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_sizes() {
        assert_eq!(primitive_size_bytes("u8"), Some(1));
        assert_eq!(primitive_size_bytes("i8"), Some(1));
        assert_eq!(primitive_size_bytes("bool"), Some(1));
        assert_eq!(primitive_size_bytes("u16"), Some(2));
        assert_eq!(primitive_size_bytes("i16"), Some(2));
        assert_eq!(primitive_size_bytes("u32"), Some(4));
        assert_eq!(primitive_size_bytes("i32"), Some(4));
        assert_eq!(primitive_size_bytes("f32"), Some(4));
        assert_eq!(primitive_size_bytes("u64"), Some(8));
        assert_eq!(primitive_size_bytes("i64"), Some(8));
        assert_eq!(primitive_size_bytes("f64"), Some(8));
        assert_eq!(primitive_size_bytes("u128"), Some(16));
        assert_eq!(primitive_size_bytes("i128"), Some(16));
        assert_eq!(primitive_size_bytes("String"), None);
        assert_eq!(primitive_size_bytes("Vec"), None);
    }
}