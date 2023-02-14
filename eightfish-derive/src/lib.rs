mod eight_fish_model;
use eight_fish_model::expand_eight_fish_model;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Provide method to build simple sql, also used to generate blockchain related data for a EF application entity.
///
/// ### Usage
///
/// ```ignore
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
/// ```ignore
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
