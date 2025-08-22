#![allow(unused_imports)]

mod dto_core;
mod eight_fish_model;

use dto_core::extract_fields_from_type;
use eight_fish_model::expand_eight_fish_model;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input,
    visit::{self, Visit},
    AngleBracketedGenericArguments, Data, DeriveInput, Field, Fields, File, GenericArgument,
    PathArguments, Type, TypePath,
};
// use syn::{
//     AngleBracketedGenericArguments, Data, DataStruct, DeriveInput, Fields, GenericArgument,
//     PathArguments, Type, TypePath,
// };

/// Provide method to build simple sql, also used to generate blockchain related data for a EF application entity.
///
/// ### Usage
///
/// ```text
/// use eightfish_derive::EightFishModel;
///
/// #[derive(Debug, Clone, Serialize, Deserialize, Default, EightFishModel)]
/// pub struct Article{
///     id: String,
///     title: String
/// }
///
/// ```
///
/// ### Generated Methods
/// ```text
/// impl Model {
///      /// get the table name of the model
///      fn model_name() -> String {
///      }
///      /// get the field names of the model, separated by commas
///      fn field_names() -> String {
///      }
///      /// get the update placeholders of the model, in format of "field1 = $1, field2 = $2"
///      fn update_placeholders() -> String {
///      }
///      /// get the select placeholders of the model, in format of "$1, $2, $3"
///      fn row_placeholders() -> String {
///      }
///      /// build a object of the struct from a row of database
///      fn from_row(row: Vec<DbValue>) -> Model {
///      }
///      /// build the sql to get a record with id
///      fn build_get_one_sql() -> String {
///      }
///      /// build the sql to get a list of records, with optional limit and offset
///      fn build_get_list_sql(limit: Option<u64>, offset: Option<u64>) -> String {
///      }
///      /// build the sql insert the record
///      fn build_insert_sql() -> String {
///      }
///      /// build the sql to update the record
///      fn build_update_sql() -> String {
///      }
///      /// build the sql to delete the record
///      fn build_delete_sql() -> String {
///      }
///      /// build the parameters for the sql statement to get a record with id
///      fn build_get_one_params(id: &str) -> Vec<ParameterValue> {
///      }
///      /// build the parameters for the sql statement to delete the record
///      fn build_delete_params(id: &str) -> Vec<ParameterValue> {
///      }
///      /// build the parameters for the sql statement to insert the record
///      fn build_insert_params(&self) -> Vec<ParameterValue> {
///      }
///      /// build the parameters for the sql statement to update the record
///      fn build_update_params(&self) -> Vec<ParameterValue> {
///      }
///      /// build both the sql statement and parameters to get a record with id
///      fn build_get_one_sql_and_params(id: &str) -> (String, Vec<ParameterValue>) {
///      }
///      /// build both the sql statement and parameters to insert the record
///      fn build_insert_sql_and_params(&self) -> (String, Vec<ParameterValue>) {
///      }
///      /// build both the sql statement and parameters to update the record
///      fn build_update_sql_and_params(&self) -> (String, Vec<ParameterValue>) {
///      }
///      /// build both the sql statement and parameters to delete a record with given id
///      fn build_delete_sql_and_params(id: &str) -> (String, Vec<ParameterValue>) {
///      }
///      /// get the id of the model object
///      fn id(&self) -> String {
///      }
///      /// calculate the hash of the model object
///      fn calc_hash(&self) -> String {
///      }
///  }
/// ```
#[proc_macro_derive(EightFishModel)]
pub fn eight_fish_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_eight_fish_model(input).into()
}

#[proc_macro_attribute]
pub fn dtocore(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_derive(EightFishDTO, attributes(dtocore))]
pub fn eight_fish_dto_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = match &input.data {
        Data::Struct(data) => {
            let fields = match &data.fields {
                Fields::Named(fields) => &fields.named,
                _ => panic!("Only named fields are supported for EightFishDTO"),
            };

            // Find the core field
            let core_field = fields
                .iter()
                .find(|field| {
                    field
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("dtocore"))
                })
                .expect("EightFishDTO requires exactly one field marked with #[dtocore]");

            // Get core field type and name
            let core_field_type = &core_field.ty;
            let core_field_name = &core_field.ident;

            // Get core type's fields
            let core_type_fields = extract_fields_from_type(core_field_type);

            // Collect non-core fields
            let other_fields: Vec<_> = fields
                .iter()
                .filter(|field| {
                    !field
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("dtocore"))
                })
                .collect();

            // Generate new struct name
            let flattened_name = format_ident!("{}Flattened", name);

            fn is_option_type(type_path: &TypePath) -> bool {
                type_path.path.segments.len() == 1 && type_path.path.segments[0].ident == "Option"
            }

            fn extract_option_inner_type(type_path: &TypePath) -> &Type {
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args, ..
                }) = &type_path.path.segments[0].arguments
                {
                    if let Some(GenericArgument::Type(inner_type)) = args.first() {
                        return inner_type;
                    }
                }
                panic!("Expected Option<T> with generic argument");
            }

            // Generate fields for the flattened struct
            let core_field_idents: Vec<_> = core_type_fields.iter().map(|f| &f.ident).collect();
            // let core_field_types: Vec<_> = core_type_fields.iter().map(|f| &f.ty).collect();
            let core_field_types: Vec<_> = core_type_fields
                .iter()
                .map(|f| {
                    let ty = &f.ty;
                    match ty {
                        Type::Path(type_path) if is_option_type(type_path) => {
                            let inner = extract_option_inner_type(type_path);
                            quote! { Option::<#inner> }
                        }
                        _ => quote! { #ty },
                    }
                })
                .collect();

            let other_field_idents: Vec<_> = other_fields.iter().map(|f| &f.ident).collect();
            // let other_field_types: Vec<_> = other_fields.iter().map(|f| &f.ty).collect();
            let other_field_types: Vec<_> = other_fields
                .iter()
                .map(|f| {
                    let ty = &f.ty;
                    match ty {
                        Type::Path(type_path) if is_option_type(type_path) => {
                            let inner = extract_option_inner_type(type_path);
                            quote! { Option::<#inner> }
                        }
                        _ => quote! { #ty },
                    }
                })
                .collect();

            let all_field_idents =
                vec![core_field_idents.clone(), other_field_idents.clone()].concat();
            let all_field_types =
                vec![core_field_types.clone(), other_field_types.clone()].concat();
            let orders = all_field_idents
                .iter()
                .enumerate()
                .map(|(i, _)| i)
                .collect::<Vec<_>>();

            let core_type_name = type_to_string(core_field_type);

            quote! {
                #[derive(Debug, Clone, Default)]
                pub struct #flattened_name {
                    #(pub #core_field_idents: #core_field_types,)*
                    #(pub #other_field_idents: #other_field_types,)*
                }

                impl #name {
                    /// build a object of the struct from a row of database
                    pub fn from_row(row: Vec<DbValue>) -> #name {
                        let mut flattened = #flattened_name::default();
                        #(
                            flattened.#all_field_idents = #all_field_types::decode(&row[#orders]).unwrap();
                        )*
                        // println!("flattened: {:?}", flattened);
                        let mut core_instance = #core_field_type::default();

                        // build a core model instance
                        #(
                            core_instance.#core_field_idents = flattened.#core_field_idents;
                        )*
                        // println!("core_instance: {:?}", core_instance);
                        // build a dto instance and return it
                        #name {
                            #core_field_name: core_instance,
                            #(#other_field_idents: flattened.#other_field_idents,)*
                        }
                    }
                }

                impl EightFishModel for #name {
                    /// get the model name of the core type
                    fn model_name(&self) -> String {
                        #core_type_name.to_string().to_lowercase()
                    }
                    /// get the id of the model object
                    fn id(&self) -> String {
                        self.#core_field_name.id()
                    }
                    /// calculate the hash of the model object
                    /// calculate the hash of the dto object
                    fn calc_hash(&self) -> String {
                        // dto's hash is the core's hash
                        self.#core_field_name.calc_hash()
                    }
                }

                // impl From<#core_field_type> for #flattened_name {
                //     fn from(core: #core_field_type) -> Self {
                //         Self {
                //             #(#core_field_idents: core.#core_field_idents,)*
                //             #(#other_field_idents: Default::default(),)*
                //         }
                //     }
                // }
            }
        }
        _ => panic!("EightFishDTO only supports structs"),
    };

    TokenStream::from(expanded)
}

fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            // Extract the path segments and join them (e.g., `std::vec::Vec` -> "Vec")
            type_path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::")
        }
        _ => "Unknown".to_string(), // Handle other types (e.g., references, tuples)
    }
}
