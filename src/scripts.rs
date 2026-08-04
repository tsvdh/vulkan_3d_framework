pub mod instance;
pub mod scene_object;

use crate::app::scene::SceneObject;
use crate::app::ui::ControlUi;
use crate::app::AppApi;
use serde::de::DeserializeOwned;
use crate::app::util::GlobalTransformData;

include!(concat!(env!("OUT_DIR"), "/get_script.rs"));

pub trait SceneObjectScript : ControlUi {

    fn frame_update(&mut self, cur_object: &mut SceneObject, global_transform_data: &GlobalTransformData, app_api: &mut AppApi);
}

pub trait InstanceScript : SceneObjectScript {

    fn test(&mut self);
}

fn convert_args<T>(args: serde_json::Value) -> T
where T: DeserializeOwned + Default
{
    if args == serde_json::Value::Null {
        return T::default()
    }
    serde_json::from_value(args).expect("Incorrect arguments for script")
}
