use crate::app::rendering::RenderItems;
use crate::app::scene::{Camera, Light, SceneEntity, SceneLayout, SceneObject, SceneTree};
use crate::app::util::CommonItems;
use egui::{collapsing_header, Align, Context, Frame, Layout, MenuBar, Panel, RichText, TextStyle, Ui, UiBuilder};
use egui_winit_vulkano::{Gui, GuiConfig};
use vulkano::image::SampleCount;
use winit::event_loop::ActiveEventLoop;

struct State {
    selected_object_id: Option<u32>,
    show_tree_panel: bool,
    show_control_panel: bool,
}

impl State {
    fn new() -> Self {
        State {
            selected_object_id: None,
            show_tree_panel: true,
            show_control_panel: false,
        }
    }
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
            state: State::new(),
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

        Panel::top("menu")
            .show_inside(&mut ui, |ui| {
                ui.menu_button("file", |ui| {

                });
            });

        Panel::left("treePanel")
            .resizable(false)
            .show_animated_inside(&mut ui, self.state.show_tree_panel, |ui| {
                ui.add_space(4.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Entity tree");
                });
                ui.separator();

                self.walk_through_tree(scene_layout, &scene_layout.scene_tree, &context, ui);
            });

        Panel::right("controlPanel")
            .resizable(false)
            .show_animated_inside(&mut ui, self.state.show_control_panel, |ui| {
                ui.add_space(4.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Entity control");
                });
                ui.separator();

                if let Some(selected_object_id) = self.state.selected_object_id {
                    scene_layout.scene_entities.get_mut(selected_object_id).ui(ui);
                } else {
                    ui.label("Nothing selected :(");
                }
            });

        Panel::top("controlBar")
            .show_separator_line(false)
            .frame(Frame::new().inner_margin(4))
            .show_inside(&mut ui, |ui| {
                MenuBar::new().ui(ui, |ui| {

                    let tree_panel_toggle_text = if self.state.show_tree_panel {
                        hex_to_emoji("23F4", 20.0)
                    } else {
                        hex_to_emoji("23F5", 20.0)
                    };
                    if ui.button(tree_panel_toggle_text).clicked() {
                        self.state.show_tree_panel = !self.state.show_tree_panel;
                    }

                    let control_panel_toggle_text = if self.state.show_control_panel {
                        hex_to_emoji("23F5", 20.0)
                    } else {
                        hex_to_emoji("23F4", 20.0)
                    };
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(control_panel_toggle_text).clicked() {
                            self.state.show_control_panel = !self.state.show_control_panel;
                            if !self.state.show_control_panel {
                                self.state.selected_object_id = None;
                            }
                        }
                    });
                });
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
                if this.state.selected_object_id.is_some_and(|id| {id == cur_object.get_id()}) {
                    this.state.selected_object_id = None;
                    this.state.show_control_panel = false;
                } else {
                    this.state.selected_object_id = Some(cur_object.get_id());
                    this.state.show_control_panel = true;
                }
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
            collapsing_header::CollapsingState::load_with_default_open(&context, header_name.into(), false)
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

fn hex_to_emoji(hex: &str, size: f32) -> RichText {
    if let Ok(number) = u32::from_str_radix(hex, 16) {
        if let Some(character) = std::char::from_u32(number) {
            return RichText::new(character).size(size)
        }
    }
    RichText::new("?").size(size)
}