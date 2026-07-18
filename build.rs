use std::{env, fs};
use std::path::Path;
use convert_case::{Case, Casing};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut output = String::new();
    output.push_str("use crate::scripts::Script;\n");
    output.push_str("\n");
    output.push_str("fn get_script(name: &str) -> Box<dyn Script> {\n");
    output.push_str("    match name {\n");
    fs::read_dir("src/scripts").unwrap()
        .map(|dir| dir.unwrap())
        .for_each(|dir| {
            let file_name_quotes = dir.file_name().to_str().unwrap().replace(".rs", "");
            let file_name = file_name_quotes.replace('"', "");
            let script_name = file_name.to_case(Case::Pascal);
            output.push_str(format!("        \"{}\" => {{ Box::new(crate::scripts::{}::{} {{}}) }}\n", file_name, file_name, script_name).as_str());
        });
    output.push_str("        _ => { panic!(\"File '{}' not found\", name) }\n");
    output.push_str("    }\n");
    output.push_str("}\n");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("get_script.rs");
    fs::write(dest_path, output).unwrap();
}