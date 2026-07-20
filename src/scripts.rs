pub mod test;
pub mod rotate;

use serde::de::DeserializeOwned;
use crate::app::scene::SceneObject;
use crate::app::script_api::AppApi;

include!(concat!(env!("OUT_DIR"), "/get_script.rs"));

pub trait Script {

    fn frame_update(&mut self, cur_object: &mut SceneObject, api: &mut AppApi);
}

fn convert_args<T>(args: serde_json::Value) -> T
where T: DeserializeOwned
{
    serde_json::from_value(args).expect("Incorrect arguments for script")
}