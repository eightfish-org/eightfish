//! Provide helper function for any user lay model
//!
//! For those function related to sql, will try to support model with following field type
//!
//!ParameterValue::Boolean(v)
//!ParameterValue::Int32(v)
//!ParameterValue::Int64(v)
//!ParameterValue::Int8(v)
//!ParameterValue::Int16(v)
//!ParameterValue::Floating32(v)
//!ParameterValue::Floating64(v)
//!ParameterValue::Uint8(v)
//!ParameterValue::Uint16(v)
//!ParameterValue::Uint32(v)
//!ParameterValue::Uint64(v)
//!ParameterValue::Str(v)
//!ParameterValue::Binary(v)
//!ParameterValue::DbNull
//!
//!
//!
//!
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
                    .map(|(i, _)| (i + 1).to_string())
                    .collect::<Vec<String>>();

                format!("{}", quote! {#($#placeholders),*})
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
                    .enumerate()
                    .map(|(i, ident)| format!("{} = ${}", ident, i + 2))
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
    let idents3 = idents.clone();
    let idents4 = idents.clone();
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
            fn get_one_sql() -> String {
                format!("SELECT {} FROM {} WHERE id = $1", #field_names, #ident_string.to_string().to_lowercase())
            }
            fn get_all_sql() -> String {
                format!("SELECT {} FROM {}", #field_names, #ident_string.to_string().to_lowercase())
            }
            fn insert_sql() -> String {
                format!("INSERT INTO {}({}) VALUES ({})", #ident_string.to_string().to_lowercase(), #field_names, #field_placeholders.to_string().replace("\"", ""))
            }
            fn update_sql() -> String {
                format!("UPDATE {} SET {} WHERE id = $1", #ident_string.to_string().to_lowercase(), #update_field_placeholders.to_string().replace("\"", ""))
            }
            fn delete_sql() -> String {
                format!("DELETE FROM {} WHERE id = $1", #ident_string.to_string().to_lowercase())
            }

            fn build_insert_param(&self) -> Vec<ParameterValue> {
                let mut param_vec: Vec<ParameterValue> = Vec::new();
                #(
                    param_vec.push( ParameterValue::Str(self.#idents3.as_str()));
                )*
                param_vec
            }
            fn build_sql_insert(&self) -> (String, Vec<ParameterValue>) {
                (Self::insert_sql(), self.build_insert_param())
            }
            fn build_update_param(&self) -> Vec<ParameterValue> {
                let mut param_vec: Vec<ParameterValue> = Vec::new();
                #(
                    param_vec.push(ParameterValue::Str(self.#idents4.as_str()));
                )*
                param_vec
            }
            fn build_sql_update(&self) -> (String, Vec<ParameterValue>) {
                (Self::update_sql(), self.build_update_param())
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
