use egui::Ui;
use crate::app::AppApi;
use crate::scripts::{convert_args, SceneObjectScript};
use log::info;
use serde::Deserialize;
use crate::app::scene::{SceneApi, SceneObject};
use crate::app::ui::ControlUi;

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

impl SceneObjectScript for Test {

    fn frame_update(&mut self, cur_object: &mut SceneObject, app_api: &mut AppApi) {
        if !self.said_hello {
            self.said_hello = true;
            info!("Hello from script!");
            info!("You said: {}", self.args.message);
        }
    }
}

impl ControlUi for Test {
    fn control_ui(&mut self, ui: &mut Ui, scene_api: &mut SceneApi) {
        ui.label("-");
    }
}