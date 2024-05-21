use darling::ast::Data;
use darling::{FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use syn::DeriveInput;
use syn::__private::quote;
use syn::__private::quote::quote;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(deref))]
struct AutoDeref {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<(), AutoDerefField>,
    #[darling(default)]
    field: Option<syn::Ident>,
    #[darling(default)]
    mutable: bool,
}

#[derive(Debug, FromField)]
struct AutoDerefField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

#[derive(Clone)]
enum DentOrIndex {
    Ident(syn::Ident),
    Index(syn::Index),
}

impl quote::ToTokens for DentOrIndex {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            DentOrIndex::Ident(ident) => ident.to_tokens(tokens),
            DentOrIndex::Index(index) => index.to_tokens(tokens),
        }
    }
}

pub(crate) fn process_auto_deref(input: DeriveInput) -> TokenStream {
    let AutoDeref {
        ident,
        generics,
        data: Data::Struct(fields),
        field,
        mutable,
    } = AutoDeref::from_derive_input(&input).unwrap()
    else {
        panic!("Failed to parse the input")
    };

    let (fd, ty) = if let Some(field) = field {
        match fields.iter().find(|f| f.ident.as_ref().unwrap() == &field) {
            Some(fd) => (DentOrIndex::Ident(ident.clone()), &fd.ty),
            None => panic!("Field {} not found", field),
        }
    } else if fields.len() == 1 {
        match fields.iter().next() {
            Some(f) => {
                let ident = match &f.ident {
                    Some(ident) => DentOrIndex::Ident(ident.clone()),
                    None => DentOrIndex::Index(syn::Index::from(0)),
                };
                (ident, &f.ty)
            }
            None => panic!("Failed to get the first field"),
        }
    } else {
        panic!("Expected a single field in the struct")
    };

    let mut code = vec![quote! {
        impl #generics std::ops::Deref for #ident #generics {
            type Target = #ty;

            fn deref(&self) -> &Self::Target {
                &self.#fd
            }
        }
    }];

    if mutable {
        code.push(quote! {
            impl #generics std::ops::DerefMut for #ident #generics {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.#fd
                }
            }
        });
    }

    quote! {
        #(#code)*
    }
}
