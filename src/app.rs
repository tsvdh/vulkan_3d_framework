mod scene;
mod logic;
mod rendering;
mod shader_modules;
mod timing;
mod ui;
mod util;
pub mod script_api;

use crate::app::logic::LogicItems;
use crate::app::rendering::RenderItems;
use crate::app::scene::{SceneLayout, SceneLayoutConfig};
use crate::app::shader_modules::fs_mod_render::RenderFragmentData;
use crate::app::shader_modules::vs_mod_render::RenderVertexData;
use crate::app::shader_modules::vs_mod_shadow::ShadowVertexData;
use crate::app::timing::TimingItems;
use crate::app::ui::GuiItems;
use crate::app::util::{get_common_vulkan_items, CommonItems, InitOption, MeshHolder};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::sync::Arc;
use std::time::Instant;
use vulkano::device::{DeviceExtensions, DeviceFeatures, QueueFlags};
use vulkano::swapchain::Surface;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use crate::app::script_api::{AppApi, LogicApi, SceneApi, TimingApi};

#[derive(Deserialize)]
pub struct Config {
    pub resolution: [u32; 2],
    pub show_frame_times: bool,
}

type UniformHolder = BTreeMap<u32, (ShadowVertexData, RenderVertexData, RenderFragmentData)>;

pub struct App {
    config: Config,
    scene_layout: SceneLayout,
    mesh_holder: MeshHolder,
    uniform_holder: UniformHolder,

    common_items: CommonItems,
    render_items: InitOption<RenderItems>,
    logic_items: LogicItems,
    gui_items: InitOption<GuiItems>,
    timing_items: TimingItems,
}

impl App {

    pub fn start() {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut app = App::new(&event_loop);
        event_loop.run_app(&mut app).unwrap();
    }

    fn new(event_loop: &EventLoop<()>) -> Self {
        let instance_extensions = Surface::required_extensions(event_loop).unwrap();
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            khr_dynamic_rendering: true,
            ..DeviceExtensions::empty()
        };
        let device_features = DeviceFeatures {
            dynamic_rendering: true,
            ..DeviceFeatures::empty()
        };

        let common_items = get_common_vulkan_items(
            Some(instance_extensions),
            Some(device_extensions),
            Some(device_features),
            QueueFlags::GRAPHICS,
            Some(event_loop)
        );

        let config: Config = serde_json::from_reader(File::open("configs/config.json").unwrap())
            .expect("Incorrect config file");
        let scene_layout_config: SceneLayoutConfig = serde_json::from_reader(File::open("configs/scene_layout.json").unwrap())
            .expect("Incorrect scene layout file");

        let (scene_layout, mesh_holder) = scene_layout_config.parse(&common_items);

        let timing_items = TimingItems::new(&config);

        App {
            common_items,
            render_items: InitOption::none(),
            logic_items: LogicItems::new(),
            gui_items: InitOption::none(),
            timing_items,
            config,
            scene_layout,
            mesh_holder,
            uniform_holder: UniformHolder::new(),
        }
    }
    
    fn get_api() -> AppApi {
        AppApi {
            logic_api: LogicApi {},
            scene_api: SceneApi {},
            timing_api: TimingApi {},
        }
    }
}

impl ApplicationHandler for App {

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Vulkan 3D Framework")
            .with_inner_size(PhysicalSize::new(self.config.resolution[0],
                                               self.config.resolution[1]))
            .with_visible(false);
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.render_items = InitOption::some(
            RenderItems::new(&self.common_items, window.clone(), &self.scene_layout)
        );
        if self.render_items.swapchain.image_count() != 2 {
            panic!("Swapchain should contain exactly two images");
        }
        self.gui_items = InitOption::some(
            GuiItems::new(event_loop, &self.common_items, &self.render_items)
        );

        // first frame render prep
        self.gui_items.build_ui(&mut self.scene_layout);
        let mut app_api = Self::get_api();
        self.logic_items.base_logic(&mut self.timing_items, &self.render_items, 
                                    &mut self.scene_layout, &mut self.uniform_holder, &mut app_api);

        window.set_visible(true);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if self.gui_items.gui.update(&event) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                self.render_items.set_recreate_swapchain(true);
            }
            WindowEvent::MouseInput {device_id: _, state: _, button: _} => {

            }
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _} => {
                self.logic_items.process_keyboard_input(event);
            }
            WindowEvent::RedrawRequested => {
                if !self.timing_items.new_frame_start(&self.logic_items) {
                    return
                }
                self.logic_items.increment_frame_id();

                // new frame start

                let gpu_prep_start = Instant::now();
                let acquire_future = match self.render_items.frame_rendering_prep(
                    &self.common_items,
                    &mut self.timing_items,
                ) {
                    None => return,
                    Some(result) => result,
                };
                self.timing_items.frame_component_durations.gpu_prep_duration = Some(gpu_prep_start.elapsed());

                self.timing_items.get_async_logic_prod().send(()).unwrap();
                *self.timing_items.get_async_cpu_start_mutex() = Instant::now();

                let render_cpu_start = Instant::now();
                self.render_items.frame_render(
                    &self.common_items,
                    &mut self.timing_items,
                    &mut self.gui_items,
                    acquire_future,
                    &self.scene_layout,
                    &self.mesh_holder,
                    &self.uniform_holder
                );
                self.timing_items.frame_component_durations.render_cpu_duration = Some(render_cpu_start.elapsed());
                *self.timing_items.get_render_gpu_start_mutex() = Instant::now();

                let ui_start = Instant::now();
                self.gui_items.build_ui(&mut self.scene_layout);
                self.timing_items.frame_component_durations.ui_duration = Some(ui_start.elapsed());

                let logic_start = Instant::now();
                let mut app_api = Self::get_api();
                self.logic_items.base_logic(
                    &mut self.timing_items,
                    &self.render_items,
                    &mut self.scene_layout,
                    &mut self.uniform_holder,
                    &mut app_api
                );
                self.timing_items.frame_component_durations.base_logic_duration = Some(logic_start.elapsed());
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.render_items.window.request_redraw();
    }
}
