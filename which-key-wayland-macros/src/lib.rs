use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericArgument, PathArguments, Type, parse_macro_input};

struct FieldAttrs {
    skip: bool,
    default_expr: Option<syn::Expr>,
    is_optional: bool,
    rename: Option<String>,
}

impl syn::parse::Parse for FieldAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut skip = false;
        let mut default_expr = None;
        let mut is_optional = false;
        let mut rename = None;

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            match ident.to_string().as_str() {
                "skip" => skip = true,
                "default" => {
                    if input.peek(syn::Token![=]) {
                        input.parse::<syn::Token![=]>()?;
                        default_expr = Some(input.parse()?);
                    }
                    is_optional = true;
                }
                "rename" => {
                    input.parse::<syn::Token![=]>()?;
                    let lit: syn::LitStr = input.parse()?;
                    rename = Some(lit.value());
                }
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unexpected node attribute `{ident}`"),
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(Self {
            skip,
            default_expr,
            is_optional,
            rename,
        })
    }
}

enum FieldKind {
    Skip,
    Arg(ArgKind),
    Block,
}

enum ArgKind {
    U32,
    I32,
    F32,
    F64,
    String,
    RcStr,
    WkColor,
    Anchor,
}

#[proc_macro_derive(KdlParse, attributes(node))]
pub fn derive_kdl_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_derive_kdl_parse(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn impl_derive_kdl_parse(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "KdlParse only supports named structs",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "KdlParse only supports structs",
            ));
        }
    };

    let pairs: Vec<_> = fields
        .iter()
        .map(generate_field)
        .collect::<syn::Result<Vec<_>>>()?;

    let all_have_defaults = pairs.iter().all(|(_, d)| d.is_some());

    let field_code: Vec<_> = pairs.iter().map(|(fc, _)| fc).collect();

    let mut expanded = quote! {
        impl crate::config::ConfigFromKdl for #name {
            fn from_kdl(doc: &kdl::KdlDocument) -> anyhow::Result<Self> {
                Ok(Self {
                    #( #field_code ,)*
                })
            }
        }
    };

    if all_have_defaults {
        let default_fields: Vec<_> = pairs.iter().map(|(_, d)| d.as_ref().unwrap()).collect();
        expanded = quote! {
            #expanded
            impl Default for #name {
                fn default() -> Self {
                    Self {
                        #( #default_fields ,)*
                    }
                }
            }
        };
    }

    Ok(expanded)
}

fn generate_field(
    field: &syn::Field,
) -> syn::Result<(proc_macro2::TokenStream, Option<proc_macro2::TokenStream>)> {
    let field_name = field.ident.as_ref().ok_or_else(|| {
        syn::Error::new_spanned(field, "KdlParse does not support unnamed fields")
    })?;

    let mut attrs = FieldAttrs {
        skip: false,
        default_expr: None,
        is_optional: false,
        rename: None,
    };
    for attr in &field.attrs {
        if attr.path().is_ident("node") {
            attrs = attr.parse_args::<FieldAttrs>()?;
            break;
        }
    }

    let default_field = if attrs.skip || attrs.is_optional {
        let default = default_expr_or_default(&attrs.default_expr);
        Some(quote!(#field_name: #default))
    } else {
        None
    };

    let kdl_name = attrs
        .rename
        .clone()
        .unwrap_or_else(|| to_kdl_name(&field_name.to_string()));

    let kind = classify_field(&attrs, &field.ty);

    let code = match kind {
        FieldKind::Skip => {
            let default = default_expr_or_default(&attrs.default_expr);
            quote!(#field_name: #default)
        }
        FieldKind::Arg(arg_kind) => generate_arg_code(field_name, &kdl_name, &arg_kind, &attrs),
        FieldKind::Block => generate_block_code(field_name, &kdl_name, &field.ty, &attrs),
    };

    Ok((code, default_field))
}

fn classify_field(attrs: &FieldAttrs, ty: &Type) -> FieldKind {
    if attrs.skip {
        return FieldKind::Skip;
    }
    match classify_arg_type(ty) {
        Some(kind) => FieldKind::Arg(kind),
        None => FieldKind::Block,
    }
}

fn classify_arg_type(ty: &Type) -> Option<ArgKind> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let last = type_path.path.segments.last()?;
    let last_name = last.ident.to_string();

    match last_name.as_str() {
        "u32" => Some(ArgKind::U32),
        "i32" => Some(ArgKind::I32),
        "f32" => Some(ArgKind::F32),
        "f64" => Some(ArgKind::F64),
        "String" => Some(ArgKind::String),
        "WkColor" => Some(ArgKind::WkColor),
        "Anchor" => Some(ArgKind::Anchor),
        "Rc" => {
            if let PathArguments::AngleBracketed(ref args) = last.arguments {
                let mut iter = args.args.iter();
                if let Some(GenericArgument::Type(Type::Path(p))) = iter.next()
                    && iter.next().is_none()
                    && p.path.is_ident("str")
                {
                    return Some(ArgKind::RcStr);
                }
            }
            None
        }
        _ => None,
    }
}

fn generate_arg_code(
    field_name: &syn::Ident,
    kdl_name: &str,
    arg_kind: &ArgKind,
    attrs: &FieldAttrs,
) -> proc_macro2::TokenStream {
    let extractor = arg_extractor(arg_kind);
    let none_branch = if attrs.is_optional {
        let default = default_expr_or_default(&attrs.default_expr);
        quote!(None => #default)
    } else {
        quote!(None => anyhow::bail!("{}: not found", #kdl_name))
    };

    quote! {
        #field_name: match doc.get_arg(#kdl_name) {
            #extractor,
            Some(_) => anyhow::bail!("{}: unexpected KDL value type", #kdl_name),
            #none_branch,
        }
    }
}

fn generate_block_code(
    field_name: &syn::Ident,
    kdl_name: &str,
    field_type: &Type,
    attrs: &FieldAttrs,
) -> proc_macro2::TokenStream {
    if attrs.is_optional {
        let default = default_expr_or_default(&attrs.default_expr);
        quote! {
            #field_name: {
                let children = doc.get(#kdl_name).and_then(|n| n.children());
                match children {
                    Some(children) => <#field_type as crate::config::ConfigFromKdl>::from_kdl(children)
                        .map_err(|e| anyhow::anyhow!("{}: {:#}", #kdl_name, e))?,
                    None => #default,
                }
            }
        }
    } else {
        quote! {
            #field_name: {
                let children = doc.get(#kdl_name)
                    .and_then(|n| n.children())
                    .ok_or_else(|| anyhow::anyhow!("{}: not found", #kdl_name))?;
                <#field_type as crate::config::ConfigFromKdl>::from_kdl(children)
                    .map_err(|e| anyhow::anyhow!("{}: {:#}", #kdl_name, e))?
            }
        }
    }
}

fn arg_extractor(kind: &ArgKind) -> proc_macro2::TokenStream {
    match kind {
        ArgKind::U32 => quote! {
            Some(kdl::KdlValue::Integer(v)) => (*v).try_into()
                .map_err(|e| anyhow::anyhow!("integer overflow: {}", e))?
        },
        ArgKind::I32 => quote! {
            Some(kdl::KdlValue::Integer(v)) => (*v).try_into()
                .map_err(|e| anyhow::anyhow!("integer overflow: {}", e))?
        },
        ArgKind::F32 => quote! {
            Some(kdl::KdlValue::Float(v)) => *v as f32
        },
        ArgKind::F64 => quote! {
            Some(kdl::KdlValue::Float(v)) => *v
        },
        ArgKind::String => quote! {
            Some(kdl::KdlValue::String(v)) => v.as_str().to_owned()
        },
        ArgKind::RcStr => quote! {
            Some(kdl::KdlValue::String(v)) => std::rc::Rc::from(v.as_str())
        },
        ArgKind::WkColor => quote! {
            Some(kdl::KdlValue::String(v)) => WkColor::from_hex(v.as_str())
                .ok_or_else(|| anyhow::anyhow!("invalid hex color"))?
        },
        ArgKind::Anchor => quote! {
            Some(kdl::KdlValue::Integer(v)) => match *v {
                1 => Anchor::union(Anchor::TOP, Anchor::RIGHT),
                2 => Anchor::union(Anchor::BOTTOM, Anchor::RIGHT),
                3 => Anchor::union(Anchor::BOTTOM, Anchor::LEFT),
                4 => Anchor::union(Anchor::TOP, Anchor::LEFT),
                n => anyhow::bail!("invalid anchor value {n}, expected 1-4"),
            }
        },
    }
}

fn to_kdl_name(rust_name: &str) -> String {
    rust_name.replace('_', "-")
}

fn default_expr_or_default(default_expr: &Option<syn::Expr>) -> proc_macro2::TokenStream {
    match default_expr {
        Some(expr) => quote!({ #expr }),
        None => quote!(Default::default()),
    }
}
