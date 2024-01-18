extern crate proc_macro;
use std::collections::HashMap;
use quote::__private::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use proc_macro::{TokenStream};
use syn::{parse_macro_input, LitStr};
use toml::Value;


fn same_type<'a, It: Iterator<Item = &'a Value>>(mut iter: impl Fn()->It) -> Option<&'a Value>{
    let itera = iter();
    let iterb = iter();
    let mut iter = iter();
    let same = itera.zip(iterb.skip(1)).map(|(a,b)|a.same_type(b)).all(|a|a);
    if let Some(s) = iter.next(){
        if same{
            return Some(s)
        }
    }
    None
}

/// Creates Rust Bindings to toml configs. Input: create_config!("path_to_file exclude_struct_1 exclude_struct_2")
/// You can exclude Structs to manually implement them. Struct start with capital names, HashMaps with lowercase. HashMaps and list can only contain one type
#[proc_macro]
pub fn create_config(input: TokenStream) -> TokenStream {
        // Parse the input TokenStream into a string literal
        let input = parse_macro_input!(input as syn::LitStr);
        let input = input.value();
        let mut input = input.split_whitespace();


        let input_str = std::fs::read_to_string(input.next().unwrap()).unwrap();
        let ignored_structs: Vec<&str> = input.collect();
        // Parse the string literal as TOML
        let parsed_toml: toml::Value = toml::from_str(&input_str).unwrap(); // Handle errors appropriately

        fn generate(top_val: &toml::Value, name: &str, structs: &mut Vec<TokenStream2>,ignored_structs: &Vec<&str>) -> TokenStream2{
            match top_val{
                toml::Value::String(s) =>quote!(String),
                toml::Value::Integer(_) => quote!(i32),
                toml::Value::Float(_) => quote!(f32),
                toml::Value::Boolean(_) => quote!(bool),
                toml::Value::Datetime(_) => todo!(),
                toml::Value::Array(a) => {
                    let typ = generate(same_type(||a.iter()).unwrap_or_else(||panic!("Array {name} has multiple value types")),"",structs,ignored_structs);
                    quote!(Box<[#typ]>)
                },
                toml::Value::Table(t) => {
                    let same_type = same_type(||t.values());
                    if name.chars().next().unwrap_or(' ').is_uppercase() || same_type.is_none(){
                        let s_name = format!("Cnfg{name}");
                        let s_name_ident = format_ident!("{s_name}");

                        if !ignored_structs.contains(&&*s_name){
                            let fields = t.iter().map(|(name,val)|{
                                let val = generate(val, name, structs,ignored_structs);
                                let name = format_ident!("{}",name);
                                quote!(#name: #val)
                            });
    
                            let struc = quote!(
                                #[derive(Deserialize,Debug)]
                                pub struct #s_name_ident{
                                    #(pub #fields,)*
                                }
                            );
                            structs.push(struc);
                        }
                     
                        quote!(#s_name_ident)
                    }else{
                        if let Some(val) = same_type{
                            let t = generate(val, name,structs,ignored_structs);
                            quote!(HashMap<String,#t>)
                        }else{
                            panic!("HashMap {name} has multiple value types");
                        }
                   
                    }
                },
            }
        }
        let mut tokens = vec![];
       generate(&parsed_toml, "",&mut tokens,&ignored_structs);
       quote!(#(#tokens)*).into()
}

fn some_kind_of_uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}


fn camel_to_snake_case(input: &str) -> String {
    let mut output = String::new();

    for (i, ch) in input.char_indices() {
        if ch.is_uppercase() {
            // Prepend an underscore if it's not the first character
            if i != 0 {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
        } else {
            output.push(ch);
        }
    }

    output
}
