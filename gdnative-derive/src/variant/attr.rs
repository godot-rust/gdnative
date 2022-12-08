macro_rules! impl_options {
    {
        self: $self:ident,
        match $ident:ident . as_str() {
            $( $name:ident, )*
            $( $alias:literal => $name_aliased:ident, )*
        }
    } => (
        match $ident.as_str() {
            $(
                stringify!($name) => {
                    $self.$name = true;
                    return Ok(());
                },
            )*
            $(
                $alias => {
                    $self.$name_aliased = true;
                    return Ok(());
                },
            )*
            _ => {},
        }
    );
    {
        self: $self:ident,
        match $ident:ident . as_str() = $lit:ident {
            $( $name:ident: $ty:ty, )*
            $( $alias:literal => $name_aliased:ident: $ty_aliased:ty, )*
        }
    } => (
        match $ident.as_str() {
            $(
                stringify!($name) => {
                    let val = match $lit {
                        syn::Lit::Str(lit_str) => lit_str.parse::<$ty>()?,
                        _ => return Err(syn::Error::new($lit.span(), "expected string literal")),
                    };

                    if $self.$name.replace(val).is_some() {
                        return Err(syn::Error::new($lit.span(), format!(
                            "the argument {} is already set",
                            stringify!($name),
                        )));
                    }

                    return Ok(());
                },
            )*
            $(
                $alias => {
                    let val = match $lit {
                        syn::Lit::Str(lit_str) => lit_str.parse::<$ty_aliased>()?,
                        _ => return Err(syn::Error::new($lit.span(), "expected string literal")),
                    };

                    if $self.$name_aliased.replace(val).is_some() {
                        return Err(syn::Error::new($lit.span(), format!(
                            "the argument {} is already set",
                            $alias,
                        )));
                    }

                    return Ok(());
                },
            )*
            _ => {},
        }
    )
}

fn generate_error_with_docs(span: proc_macro2::Span, message: &str) -> syn::Error {
    syn::Error::new(
        span,
        format!(
            "{message}\n\texpecting #[variant(...)]. See documentation:\n\thttps://docs.rs/gdnative/0.9.0/gdnative/core_types/trait.ToVariant.html#field-attributes"
        ),
    )
}

pub trait AttrBuilder: FromIterator<syn::Meta> {
    type Attr;
    fn done(self) -> Result<Self::Attr, syn::Error>;
}

pub mod field;
pub mod item;

pub use field::{FieldAttr, FieldAttrBuilder};
pub use item::{ItemAttr, ItemAttrBuilder};
