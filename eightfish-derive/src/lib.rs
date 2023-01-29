use eightfish::EightFishModel;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, FieldsNamed};

#[proc_macro_derive(EightFishModel)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let field_names = match data {
        syn::Data::Struct(ref s) => match s.fields {
            syn::Fields::Named(FieldsNamed { ref named, .. }) => {
                let idents = named.iter().map(|f| &f.ident);
                format!("{}", quote! {#(#idents),*})
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let field_placeholders = match data {
        syn::Data::Struct(ref s) => match s.fields {
            syn::Fields::Named(FieldsNamed { ref named, .. }) => {
                let placeholders: Vec<String> = named
                    .iter()
                    .enumerate()
                    .map(|(i, _)| "?".to_string())
                    .collect::<Vec<String>>();

                format!("{}", quote! {#(#placeholders),*})
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let update_field_placeholders = match data {
        syn::Data::Struct(ref s) => match s.fields {
            syn::Fields::Named(FieldsNamed { ref named, .. }) => {
                let placeholders: Vec<String> = named
                    .iter()
                    .map(|f| &f.ident)
                    .filter_map(|ident| Some(ident.clone().unwrap().to_string()))
                    .map(|ident| format!("{} = ?", ident))
                    .collect::<Vec<String>>();

                format!("{}", quote! {#(#placeholders),*})
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let idents = match data {
        syn::Data::Struct(ref s) => match s.fields {
            syn::Fields::Named(FieldsNamed { ref named, .. }) => {
                let idents = named.iter().map(|f| &f.ident);
                idents
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };
    let idents2 = idents.clone();

    let orders = match data {
        syn::Data::Struct(ref s) => match s.fields {
            syn::Fields::Named(FieldsNamed { ref named, .. }) => {
                named.iter().enumerate().map(|(i, _)| i + 1)
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };
    let ident_string = ident.to_string();
    let output = quote! {
        impl #ident {
            fn model_name() -> String {
                #ident_string.to_string().to_lowercase()
            }
            fn field_names() -> String {
                format!("{}", #field_names)
            }
            fn update_placeholders() -> String {
                #update_field_placeholders.to_string().replace("\"", "")
            }
            fn row_placeholders() -> String {
                #field_placeholders.to_string().replace("\"", "")
            }
            fn to_vec(&self) -> Vec<String> {
                let mut field_vec: Vec<String> = Vec::new();
                #(
                    field_vec.push(self.#idents.to_string());
                )*
                field_vec
            }
            fn to_row(&self, hash: String) -> Vec<String> {
                let mut row = self.to_vec();
                row.insert(0, hash);
                row
            }
            fn from_row(row: Vec<String>) -> #ident {
                let mut settings = #ident::default();
                #(
                    settings.#idents2 = row[#orders].to_string();
                )*
                settings
            }
            fn get_hash_from_row(row: Vec<String>) -> String {
                row[0].to_string()
            }
        }
        impl EightFishModel for #ident {
            fn id(&self) -> String {
                self.id.clone()
            }

            fn calc_hash(&self) -> String {
                let json_val= serde_json::to_value(self).unwrap();
                let digest = json_digest::digest_data(&json_val).unwrap();
                digest
            }
        }
    };

    output.into()
}
