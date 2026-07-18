use std::{env, fs};
use std::path::Path;
use codegen::{Block, Scope};
use convert_case::{Case, Casing};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut block = Block::new("match name");
    fs::read_dir("src/scripts").unwrap()
        .map(|dir| dir.unwrap())
        .for_each(|dir| {
            let file_name = dir.file_name().to_str().unwrap()
                .replace(".rs", "")
                .replace('"', "");
            let script_name = file_name.to_case(Case::Pascal);

            block.line(format!("\"{}\" => {{ Box::new(crate::scripts::{}::{} {{}}) }}", file_name, file_name, script_name).as_str());
        });
    block.line("_ => { panic!(\"File '{}' not found\", name); }");

    let mut scope = Scope::new();
    scope.import("crate::scripts", "Script");
    scope.new_fn("get_script")
        .arg("name", "&str")
        .ret("Box<dyn Script>")
        .push_block(block);

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("get_script.rs");
    fs::write(dest_path, scope.to_string()).unwrap();
}