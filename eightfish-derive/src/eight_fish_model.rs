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
        .map(|(i, ident)| format!("{} = ${}", quote! {#ident}, i + 1))
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
            Type::Path(type_path) if type_path.clone().into_token_stream().to_string() == "i16" => {
                quote! {
                    param_vec.push(ParameterValue::Int16(self.#field_name));
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
            /// get the table name of the model
            pub fn model_name() -> String {
                #ident_string.to_string().to_lowercase()
            }
            /// get the field names of the model, separated by commas
            pub fn field_names() -> String {
                format!("{}", #field_names)
            }
            /// get the update placeholders of the model, in format of "field1 = $1, field2 = $2"
            pub fn update_placeholders() -> String {
                #update_field_placeholders.to_string().replace("\"", "")
            }
            /// get the select placeholders of the model, in format of "$1, $2, $3"
            pub fn row_placeholders() -> String {
                #field_placeholders.to_string().replace("\"", "")
            }
            /// build a object of the struct from a row of database
            pub fn from_row(row: Vec<DbValue>) -> #ident {
                let mut settings = #ident::default();
                #(
                    settings.#field_identifiers_2 = #types::decode(&row[#orders]).unwrap();
                )*
                settings
            }
            /// build the sql to get a record with id
            pub fn build_get_one_sql() -> String {
                format!("SELECT {} FROM {} WHERE id = $1", #field_names, #ident_string.to_string().to_lowercase())
            }

            /// build the sql to get a record with id
            pub fn build_get_one_by_sql(column: &str) -> String {
                format!("SELECT {} FROM {} WHERE {} = $1", #field_names, #ident_string.to_string().to_lowercase(), column)
            }

            /// build the sql to get a list of records, with optional limit and offset
            pub fn build_get_list_sql(limit: u64, offset: u64) -> String {
                let query = format!("SELECT {} FROM {} LIMIT {} OFFSET {}", #field_names, #ident_string.to_string().to_lowercase(), limit, offset);
                query
            }

            /// build the sql to get a list of records, with optional limit and offset
            pub fn build_get_list_by_sql(column: &str, limit: u64, offset: u64) -> String {
                let query = format!("SELECT {} FROM {} WHERE {} = $1 LIMIT {} OFFSET {}", #field_names, #ident_string.to_string().to_lowercase(), column, limit, offset);
                query
            }

            /// build the sql insert the record
            pub fn build_insert_sql() -> String {
                format!("INSERT INTO {}({}) VALUES ({})", #ident_string.to_string().to_lowercase(), #field_names, #field_placeholders.to_string().replace("\"", ""))
            }
            /// build the sql to update the record
            pub fn build_update_sql() -> String {
                format!("UPDATE {} SET {} WHERE id = $1", #ident_string.to_string().to_lowercase(), #update_field_placeholders.to_string().replace("\"", ""))
            }
            /// build the sql to delete the record
            pub fn build_delete_sql() -> String {
                format!("DELETE FROM {} WHERE id = $1", #ident_string.to_string().to_lowercase())
            }
            /// build the parameters for the sql statement to get a record with id
            pub fn build_get_list_by_params(value: &str) -> Vec<ParameterValue> {
                let mut param_vec: Vec<ParameterValue> = Vec::new();
                param_vec.push(ParameterValue::Str(value));
                param_vec
            }
            /// build the parameters for the sql statement to get a record with id
            pub fn build_get_one_params(value: &str) -> Vec<ParameterValue> {
                let mut param_vec: Vec<ParameterValue> = Vec::new();
                param_vec.push(ParameterValue::Str(value));
                param_vec
            }
            /// build the parameters for the sql statement to delete the record
            pub fn build_delete_params(id: &str) -> Vec<ParameterValue> {
                let mut param_vec: Vec<ParameterValue> = Vec::new();
                param_vec.push(ParameterValue::Str(id));
                param_vec
            }
            /// build the parameters for the sql statement to insert the record
            pub fn build_insert_params(&self) -> Vec<ParameterValue> {
                let mut param_vec: Vec<ParameterValue> = Vec::new();
                #(
                    #create_params
                )*
                param_vec
            }
            /// build the parameters for the sql statement to update the record
            pub fn build_update_params(&self) -> Vec<ParameterValue> {
                let mut param_vec: Vec<ParameterValue> = Vec::new();
                #(
                    #update_params
                )*
                param_vec
            }

            /// build both the sql statement and parameters to get a list record with column
            pub fn build_get_list_by_sql_and_params<'a, 'b>(column: &'a str, value: &'b str, limit: u64, offset: u64) -> (String, Vec<ParameterValue<'b>>) {
                let sql = Self::build_get_list_by_sql(column, limit, offset);
                let params = Self::build_get_one_params(value);
                (sql, params)
            }
            /// build both the sql statement and parameters to get a record with id
            pub fn build_get_one_sql_and_params(value: &str) -> (String, Vec<ParameterValue>) {
                (Self::build_get_one_sql(), Self::build_get_one_params(value))
            }
            /// build both the sql statement and parameters to get a record with id
            pub fn build_get_one_by_sql_and_params<'a, 'b>(column: &'a str, value: &'b str) -> (String, Vec<ParameterValue<'b>>) {
                let sql = Self::build_get_one_by_sql(column);
                let params = Self::build_get_one_params(value);
                (sql, params)
            }
            /// build both the sql statement and parameters to insert the record
            pub fn build_insert_sql_and_params(&self) -> (String, Vec<ParameterValue>) {
                (Self::build_insert_sql(), self.build_insert_params())
            }
            /// build both the sql statement and parameters to update the record
            pub fn build_update_sql_and_params(&self) -> (String, Vec<ParameterValue>) {
                (Self::build_update_sql(), self.build_update_params())
            }
            /// build both the sql statement and parameters to delete a record with given id
            pub fn build_delete_sql_and_params(id: &str) -> (String, Vec<ParameterValue>) {
                (Self::build_delete_sql(), Self::build_delete_params(id))
            }
        }
        impl EightFishModel for #ident {
            /// get the id of the model object
            fn id(&self) -> String {
                self.id.clone()
            }
            /// calculate the hash of the model object
            fn calc_hash(&self) -> String {
                let json_val= serde_json::to_value(self).unwrap();
                let digest = json_digest::digest_data(&json_val).unwrap();
                digest
            }
        }
    };

    output
}
