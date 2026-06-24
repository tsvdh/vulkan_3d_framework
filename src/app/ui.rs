use crate::app::rendering::RenderItems;
use crate::app::scene::{Camera, Light, SceneEntity, SceneLayout, SceneObject, SceneTree};
use crate::app::util::CommonItems;
use egui::{Context, TextStyle, Ui, UiBuilder};
use egui_winit_vulkano::{Gui, GuiConfig};
use vulkano::image::SampleCount;
use winit::event_loop::ActiveEventLoop;

#[derive(Default)]
struct State {
    selected_object_id: Option<u32>
}

pub struct GuiItems {
    // public
    pub gui: Gui,

    // configuration

    // access through methods

    // private
    state: State,
}

impl GuiItems {

    pub fn new(event_loop: &ActiveEventLoop,
               vulkan_items: &CommonItems,
               render_items: &RenderItems,
    ) -> GuiItems
    {
        let swapchain = render_items.swapchain.clone();
        let egui_config = GuiConfig {
            allow_srgb_render_target: true,
            is_overlay: true,
            samples: SampleCount::Sample1,
        };
        
        let gui = Gui::new(
            event_loop,
            swapchain.surface().clone(),
            vulkan_items.queue.clone(),
            swapchain.image_format(),
            egui_config
        );
        
        GuiItems {
            gui,
            state: Default::default(),
        }
    }

    fn set_font_sizes(context: &Context) {
        context.global_style_mut(|style| {
            for (text_style, font_id) in style.text_styles.iter_mut() {
                match text_style {
                    TextStyle::Body => { font_id.size = 15.0 }
                    TextStyle::Button => { font_id.size = 15.0 }
                    TextStyle::Heading => { font_id.size = 22.0 }
                    _ => {}
                }
            }
        });
    }

    pub fn build_ui(&mut self,
                    scene_layout: &mut SceneLayout,
    ) {
        self.gui.begin_frame();
        let context = self.gui.context();
        Self::set_font_sizes(&context);

        let mut ui = Ui::new(context.clone(), "ui".into(), UiBuilder::new());

        // egui::Panel::top("topPanel").show_inside(&mut ui, |ui| {
        //     egui::Button::selectable(true, "fileSelect").
        // });

        egui::Panel::left("treePanel")
            .resizable(false)
            .show_inside(&mut ui, |ui| {
                ui.add_space(4.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Entity tree");
                });
                ui.separator();

                self.walk_through_tree(scene_layout, &scene_layout.scene_tree, &context, ui);
            });

        egui::Panel::right("controlPanel")
            .resizable(false)
            .show_inside(&mut ui, |ui| {
                ui.add_space(4.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Entity control")
                });
                ui.separator();

                if let Some(selected_object_id) = self.state.selected_object_id {
                    scene_layout.scene_entities.get_mut(selected_object_id).ui(ui);
                } else {
                    ui.label("Nothing selected :(");
                }
            });
    }

    fn walk_through_tree(&mut self,
                         scene_layout: &SceneLayout, scene_tree: &SceneTree,
                         context: &Context, ui: &mut Ui)
    {
        let cur_object = scene_layout.scene_entities.get(scene_tree.entity_id);
        let header_name = format!("{}_header", cur_object.get_name());

        let show_item_label = |this: &mut GuiItems, ui: &mut Ui| {
            let object_selected = this.state.selected_object_id.is_some_and(
                |id| { id == cur_object.get_id() });
            if ui.selectable_label(object_selected, cur_object.get_name()).clicked() {
                this.state.selected_object_id = Some(cur_object.get_id());
            }
        };

        let show_children = |this: &mut GuiItems, ui: &mut Ui| {
            for child_tree in scene_tree.children.iter() {
                this.walk_through_tree(scene_layout, child_tree, context, ui);
            }
        };

        if cur_object.get_name() == "root" {
            show_children(self, ui);
            return
        }

        if scene_tree.children.is_empty() {
            show_item_label(self, ui);
        } else {
            egui::collapsing_header::CollapsingState::load_with_default_open(&context, header_name.into(), false)
                .show_header(ui, |ui| {
                    show_item_label(self, ui);
                })
                .body(|ui| {
                    show_children(self, ui);
                });
        }
    }
}

pub trait ControlUi {
    fn ui(&mut self, ui: &mut Ui);
}

impl ControlUi for Light {
    fn ui(&mut self, ui: &mut Ui) {
        ui.label("It's the light!");
    }
}
impl ControlUi for Camera {
    fn ui(&mut self, ui: &mut Ui) {
        ui.label("It's the camera!");
    }
}
impl ControlUi for SceneObject {
    fn ui(&mut self, ui: &mut Ui) {
        ui.label(format!("It's object {}!", self.name));
    }
}

impl ControlUi for Box<dyn SceneEntity> {
    fn ui(&mut self, ui: &mut Ui) {
        self.as_mut().ui(ui)
    }
}
