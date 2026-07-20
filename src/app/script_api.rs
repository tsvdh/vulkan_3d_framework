use std::collections::BTreeSet;
use winit::keyboard::KeyCode;

pub struct AppApi {
    pub logic_api: LogicApi,
    pub scene_api: SceneApi,
    pub timing_api: TimingApi,
}

pub struct LogicApi {
    pub keys_pressed: BTreeSet<KeyCode>,
    pub keys_down: BTreeSet<KeyCode>,
}

pub struct SceneApi {

}

pub struct TimingApi {
    pub frame_duration: f32
}
