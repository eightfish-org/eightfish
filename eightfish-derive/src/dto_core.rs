use syn::{DeriveInput, Data, Fields, Field, Type, File, visit::{self, Visit}};
use quote::quote;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use cargo_metadata::MetadataCommand;

pub fn extract_fields_from_type(ty: &Type) -> Vec<Field> {
    // Assuming the type is a named struct accessible in the current scope
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;
            // Here we're assuming the type name is the last segment of the path
            if let Some(segment) = path.segments.last() {
                let ident = &segment.ident;

                let struct_def = find_struct_definition(&ident.to_string()).expect("Could not find inner struct definition");
                
                // Check if the parsed item is actually a struct
                if let Data::Struct(data) = struct_def.data {
                    match data.fields {
                        Fields::Named(fields) => fields.named.into_iter().collect(),
                        _ => Vec::new(),  // If not named fields, return an empty vec
                    }
                } else {
                    Vec::new()  // If it's not a struct, return an empty vec
                }
            } else {
                Vec::new()
            }
        },
        _ => Vec::new(), // If the type is not a path (struct), return an empty vec
    }
}


// Helper to find struct definition in either solo package or across workspace
fn find_struct_definition(struct_name: &str) -> Option<DeriveInput> {
    match is_workspace() {
        true => find_struct_in_workspace(struct_name),
        false => find_struct_in_solo_package(struct_name),
    }
}

// Check if we're in a workspace
fn is_workspace() -> bool {
    let metadata = MetadataCommand::new().no_deps().exec().expect("Failed to get cargo metadata");
    println!(" members {}", metadata.workspace_members.len());
    metadata.workspace_members.len() > 1
}

// Search for struct in a workspace
fn find_struct_in_workspace(struct_name: &str) -> Option<DeriveInput> {
    let metadata = MetadataCommand::new().no_deps().exec().expect("Failed to get cargo metadata");

    for package in metadata.workspace_packages() {
        println!("package.manifest_path {}", package.manifest_path);

        if let Some(src_path) = package.manifest_path.parent() {
            if let Some(struct_def) = find_in_src_and_tests(&src_path.to_string(), struct_name) {
                return Some(struct_def);
            }
        }
    }
    None
}

// Search for struct in a solo package
fn find_struct_in_solo_package(struct_name: &str) -> Option<DeriveInput> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    println!("manifest_dir {}", manifest_dir);
    find_in_src_and_tests(&manifest_dir, struct_name)
}

fn find_in_src_and_tests(src_path: &str, struct_name: &str) -> Option<DeriveInput> {
    let ret = search_directory_for_struct(&Path::new(&src_path).join("src"), struct_name);
    if ret.is_none() {
        search_directory_for_struct(&Path::new(&src_path).join("tests"), struct_name)
    } else {
        ret
    }
}

// General function to search for struct in a directory
fn search_directory_for_struct(src_dir: &Path, struct_name: &str) -> Option<DeriveInput> {
    println!("src_dir, struct_name: {:?} {}", src_dir, struct_name);
    for entry in WalkDir::new(src_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file() && e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            if let Ok(file) = syn::parse_file(&content) {
                if let Some(struct_def) = find_struct_in_file(&file, struct_name) {
                    return Some(struct_def);
                }
            }
        }
    }
    None
}

fn find_struct_in_file(file: &File, struct_name: &str) -> Option<DeriveInput> {
    struct StructFinder {
        found: Option<DeriveInput>,
        name: String,
    }

    impl<'ast> Visit<'ast> for StructFinder {
        fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
            if node.ident == self.name {
                self.found = Some(syn::parse2(quote! { #node }).unwrap());
            }
            visit::visit_item_struct(self, node);
        }
    }

    let mut finder = StructFinder {
        found: None,
        name: struct_name.to_string(),
    };
    finder.visit_file(file);
    finder.found
}
