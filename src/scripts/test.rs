use crate::app::scene::SceneObject;
use crate::app::AppApi;
use crate::scripts::{convert_args, Script};
use log::info;
use serde::Deserialize;

#[derive(Deserialize)]
struct Args {
    message: String,
}

pub struct Test {
    args: Args,
    said_hello: bool,
}

impl Test {
    pub fn new(args: serde_json::Value) -> Self {
        Test {
            args: convert_args(args),
            said_hello: false,
        }
    }
}

impl Script for Test {

    fn frame_update(&mut self, cur_object: &mut SceneObject, app_api: &mut AppApi) {
        if !self.said_hello {
            self.said_hello = true;
            info!("Hello from script!");
            info!("You said: {}", self.args.message);
        }
    }
}