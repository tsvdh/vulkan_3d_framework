use crate::app::rendering::RenderItems;
use crate::app::scene::{Camera, Light, SceneEntity, SceneLayout, SceneObject, SceneTree};
use crate::app::util::CommonItems;
use egui::{collapsing_header, Align, Atoms, Context, DragValue, Frame, Layout, MenuBar, Panel, RichText, TextStyle, Ui, UiBuilder};
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

const TEXT_SIZE: f32 = 15.0;
const HEADER_SIZE: f32 = 22.0;

const LEFT_FOLD_ICON_HEX: &str = "23F4";
const RIGHT_FOLD_ICON_HEX: &str = "23F5";
const CAMERA_ICON_HEX: &str = "1F3A5";
const LIGHT_ICON_HEX: &str = "2600";
const OBJECT_ICON_HEX: &str = "1F4BC";

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
                    TextStyle::Body => { font_id.size = TEXT_SIZE }
                    TextStyle::Button => { font_id.size = TEXT_SIZE }
                    TextStyle::Heading => { font_id.size = HEADER_SIZE }
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
                ui.menu_button("file", |_ui| {

                });
            });

        Panel::left("treePanel")
            .resizable(false)
            .show_animated_inside(&mut ui, self.state.show_tree_panel, |ui| {
                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Entity tree");
                });
                ui.separator();

                self.walk_through_tree(scene_layout, &scene_layout.scene_tree_root, &context, ui);
            });

        Panel::right("controlPanel")
            .resizable(false)
            .show_animated_inside(&mut ui, self.state.show_control_panel, |ui| {
                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Entity control");
                });
                ui.separator();

                if let Some(selected_object_id) = self.state.selected_object_id {
                    scene_layout.scene_entities.get_mut(selected_object_id).control_ui(ui);
                } else {
                    ui.label("Nothing selected");
                }
            });

        Panel::top("foldBar")
            .show_separator_line(false)
            .frame(Frame::new().inner_margin(4))
            .show_inside(&mut ui, |ui| {
                MenuBar::new().ui(ui, |ui| {

                    let tree_panel_toggle_text = if self.state.show_tree_panel {
                        hex_to_emoji(LEFT_FOLD_ICON_HEX, 20.0)
                    } else {
                        hex_to_emoji(RIGHT_FOLD_ICON_HEX, 20.0)
                    };
                    if ui.button(tree_panel_toggle_text).clicked() {
                        self.state.show_tree_panel = !self.state.show_tree_panel;
                    }

                    let control_panel_toggle_text = if self.state.show_control_panel {
                        hex_to_emoji(RIGHT_FOLD_ICON_HEX, 20.0)
                    } else {
                        hex_to_emoji(LEFT_FOLD_ICON_HEX, 20.0)
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

            if ui.selectable_label(object_selected, cur_object.tree_heading_atoms()).clicked() {
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

pub fn vec3_drag_values_float_range(ui: &mut Ui, vec3: &mut [f32; 3], speed: f32, min: f32, max: f32)
{
    ui.horizontal(|ui| {
        ui.label("x: ");
        ui.add(DragValue::new(&mut vec3[0]).speed(speed).range(min..=max));
        ui.label("y: ");
        ui.add(DragValue::new(&mut vec3[1]).speed(speed).range(min..=max));
        ui.label("z: ");
        ui.add(DragValue::new(&mut vec3[2]).speed(speed).range(min..=max));
    });
}
pub fn vec3_drag_values_int_range(ui: &mut Ui, vec3: &mut [f32; 3], speed: f32, min: i32, max: i32) {
    vec3_drag_values_float_range(ui, vec3, speed, min as f32, max as f32);
}
pub fn vec3_drag_values(ui: &mut Ui, vec3: &mut [f32; 3], speed: f32) {
    vec3_drag_values_int_range(ui, vec3, speed, i32::MIN, i32::MAX);
}

pub trait ControlUi {
    fn control_ui(&mut self, ui: &mut Ui);
}

impl ControlUi for Light {
    fn control_ui(&mut self, ui: &mut Ui, ) {
        ui.horizontal(|ui| {
            ui.label(hex_to_emoji(LIGHT_ICON_HEX, 20.0));
            ui.label(self.get_name());
        });
        ui.separator();
        match self {
            Light::Point { id: _, position } => {
                ui.label("Position");
                vec3_drag_values(ui, position.as_mut(), 0.1);

            }
            Light::Directional { id: _, direction } => {
                ui.label("Direction");
                let old_direction = direction.clone();
                vec3_drag_values(ui, direction.as_mut(), 0.01);
                if direction.length() == 0.0 {
                    *direction = old_direction;
                } else {
                    *direction = direction.normalize();
                }
            }
        }
    }
}
impl ControlUi for Camera {
    fn control_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(hex_to_emoji(CAMERA_ICON_HEX, 20.0));
            ui.label(self.get_name());
        });
        ui.separator();
        ui.label("Position");
        vec3_drag_values(ui, &mut self.position.as_mut(), 0.1);
        ui.add_space(8.0);
        ui.label("Horizon");
        vec3_drag_values(ui, &mut self.horizon.as_mut(), 0.1);
    }
}
impl ControlUi for SceneObject {
    fn control_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(hex_to_emoji(OBJECT_ICON_HEX, 20.0));
            ui.label(format!("{} (Object)", self.get_name()))
        });
        ui.separator();
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Transforms").size(TEXT_SIZE + 2.0))
        });
        ui.label("Translation");
        vec3_drag_values(ui, &mut self.translation.as_mut(), 0.1);
        ui.add_space(8.0);
        ui.label("Rotation");
        vec3_drag_values(ui, &mut self.rotation.as_mut(), 1.0);
        ui.add_space(8.0);
        ui.label("Scale");
        vec3_drag_values(ui, &mut self.scale.as_mut(), 0.1);
        if self.mesh_id.is_some() {
            ui.separator();
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Mesh").size(TEXT_SIZE + 2.0))
            });
            ui.label("-");
        }
        if let Some(material) = self.material.as_mut() {
            ui.separator();
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Material").size(TEXT_SIZE + 2.0))
            });
            material.control_ui(ui);
        }
    }
}
impl ControlUi for Box<dyn SceneEntity> {
    fn control_ui(&mut self, ui: &mut Ui) {
        self.as_mut().control_ui(ui)
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

pub trait TreeHeadingUi {
    fn tree_heading_atoms(&'_ self) -> Atoms<'_>;
}
impl TreeHeadingUi for Camera {
    fn tree_heading_atoms(&'_ self) -> Atoms<'_> {
        Atoms::new((hex_to_emoji(CAMERA_ICON_HEX, TEXT_SIZE), "Camera"))
    }
}
impl TreeHeadingUi for Light {
    fn tree_heading_atoms(&'_ self) -> Atoms<'_> {
        Atoms::new((hex_to_emoji(LIGHT_ICON_HEX, TEXT_SIZE), "Light"))
    }
}
impl TreeHeadingUi for SceneObject {
    fn tree_heading_atoms(&'_ self) -> Atoms<'_> {
        Atoms::new((hex_to_emoji(OBJECT_ICON_HEX, TEXT_SIZE), self.get_name()))
    }
}
impl TreeHeadingUi for Box<dyn SceneEntity> {
    fn tree_heading_atoms(&'_ self) -> Atoms<'_> {
        self.as_ref().tree_heading_atoms()
    }
}