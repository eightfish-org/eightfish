//! Provide helper function for any application model
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
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DataStruct, DeriveInput, Fields, Type};

pub fn expand_eight_fish_model(input: DeriveInput) -> TokenStream {
    let DeriveInput { ident, data, .. } = input;
    let fields = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.named,
        _ => panic!("This derive macro only works on structs with named fields"),
    };

    let field_identifiers = fields.iter().map(|f| &f.ident);
    let field_identifiers_for_names = field_identifiers.clone();
    let field_names = format!("{}", quote! {#(#field_identifiers_for_names),*});

    let field_placeholders = fields
        .iter()
        .enumerate()
        .map(|(i, _)| format!("${}", i + 1))
        .collect::<Vec<String>>()
        .join(", ");

    let update_field_placeholders = field_identifiers
        .clone()
        .enumerate()
        .map(|(i, ident)| format!("{} = ${}", quote! {#ident}, i + 2))
        .collect::<Vec<String>>()
        .join(", ");

    let types = fields.iter().map(|f| &f.ty);
    let field_identifiers_2 = field_identifiers.clone();
    let orders = fields.iter().enumerate().map(|(i, _)| i);
    let ident_string = ident.to_string();
    let create_params = fields.clone().into_iter().map(|f| {
        let field_name = f.ident;
        match f.ty {
            Type::Path(type_path)
                if type_path.clone().into_token_stream().to_string() == "String" =>
            {
                quote! {
                    param_vec.push(ParameterValue::Str(self.#field_name.as_str()));
                }
            }
            Type::Path(type_path) if type_path.clone().into_token_stream().to_string() == "i64" => {
                quote! {
                    param_vec.push(ParameterValue::Int64(self.#field_name));
                }
            }
            Type::Path(type_path)
                if type_path.clone().into_token_stream().to_string() == "bool" =>
            {
                quote! {
                    param_vec.push(ParameterValue::Boolean(self.#field_name));
                }
            }
            _ => unimplemented!(),
        }
    });
    let update_params = create_params.clone();
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

            fn from_row(row: Vec<DbValue>) -> #ident {
                let mut settings = #ident::default();
                #(
                    settings.#field_identifiers_2 = #types::decode(&row[#orders]).unwrap();
                )*
                settings
            }

            fn build_get_one_sql() -> String {
                format!("SELECT {} FROM {} WHERE id = $1", #field_names, #ident_string.to_string().to_lowercase())
            }
            fn build_get_all_sql(limit: Option<u64>, offset: Option<u64>) -> String {
                let query = format!("SELECT {} FROM {}", #field_names, #ident_string.to_string().to_lowercase());
                match (limit, offset) {
                    (Some(l), Some(o)) => format!("{} LIMIT {} OFFSET {}", query, l, o),
                    (Some(l), None) => format!("{} LIMIT {}", query, l),
                    _ => query
                }
            }
            fn build_insert_sql() -> String {
                format!("INSERT INTO {}({}) VALUES ({})", #ident_string.to_string().to_lowercase(), #field_names, #field_placeholders.to_string().replace("\"", ""))
            }
            fn build_update_sql() -> String {
                format!("UPDATE {} SET {} WHERE id = $1", #ident_string.to_string().to_lowercase(), #update_field_placeholders.to_string().replace("\"", ""))
            }
            fn build_delete_sql() -> String {
                format!("DELETE FROM {} WHERE id = $1", #ident_string.to_string().to_lowercase())
            }
            fn build_get_one_params(id: &str) -> Vec<ParameterValue> {
                let mut param_vec: Vec<ParameterValue> = Vec::new();
                param_vec.push(ParameterValue::Str(id));
                param_vec
            }
            fn build_delete_params(id: &str) -> Vec<ParameterValue> {
                let mut param_vec: Vec<ParameterValue> = Vec::new();
                param_vec.push(ParameterValue::Str(id));
                param_vec
            }
            fn build_insert_params(&self) -> Vec<ParameterValue> {
                let mut param_vec: Vec<ParameterValue> = Vec::new();
                #(
                    #create_params
                )*
                param_vec
            }
            fn build_update_params(&self) -> Vec<ParameterValue> {
                let mut param_vec: Vec<ParameterValue> = Vec::new();
                #(
                    #update_params
                )*
                param_vec
            }
            fn build_get_one_sql_and_params(id: &str) -> (String, Vec<ParameterValue>) {
                (Self::build_get_one_sql(), Self::build_get_one_params(id))
            }
            fn build_delete_sql_and_params(id: &str) -> (String, Vec<ParameterValue>) {
                (Self::build_delete_sql(), Self::build_delete_params(id))
            }
            fn build_insert_sql_and_params(&self) -> (String, Vec<ParameterValue>) {
                (Self::build_insert_sql(), self.build_insert_params())
            }

            fn build_update_sql_and_params(&self) -> (String, Vec<ParameterValue>) {
                (Self::build_update_sql(), self.build_update_params())
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

    output
}
