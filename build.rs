use std::{env, fs};
use std::path::Path;
use codegen::{Block, Scope};
use convert_case::{Case, Casing};

const SCRIPT_TYPES: [&str; 2] = ["instance", "scene_object"];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/scripts");

    let mut scope = Scope::new();

    for script_type in SCRIPT_TYPES {
        let mut block = Block::new("match name");
        fs::read_dir(format!("src/scripts/{}", script_type)).unwrap()
            .map(|dir| dir.unwrap())
            .for_each(|dir| {
                let file_name = dir.file_name().to_str().unwrap()
                    .replace(".rs", "")
                    .replace('"', "");
                let script_name = file_name.to_case(Case::Pascal);

                block.line(format!("\"{}\" => {{ Box::new(crate::scripts::{}::{}::{}::new(args)) }}",
                                   file_name, script_type, file_name, script_name).as_str());
            });
        block.line("_ => { panic!(\"File '{}' not found\", name); }");

        scope.new_fn(format!("get_{}_script", script_type))
            .vis("pub")
            .arg("name", "&str")
            .arg("args", "serde_json::Value")
            .ret(format!("Box<dyn {}Script>", script_type.to_case(Case::Pascal)))
            .push_block(block);
    }

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("get_script.rs");
    fs::write(dest_path, scope.to_string()).unwrap();
}