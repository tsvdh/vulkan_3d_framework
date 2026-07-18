mod app;
mod scripts;

use crate::app::App;

include!(concat!(env!("OUT_DIR"), "/get_script.rs"));

fn main() {
    get_script("script_c");
}