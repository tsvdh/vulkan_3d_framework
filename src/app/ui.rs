use egui::{Context, TextStyle, Ui, UiBuilder};
use crate::app::rendering::RenderItems;
use crate::app::util::CommonItems;
use egui_winit_vulkano::{Gui, GuiConfig};
use log::info;
use vulkano::image::SampleCount;
use winit::event_loop::ActiveEventLoop;
use crate::app::scene::{SceneLayout, SceneTree};

#[derive(Default)]
struct State {
    camera_selected: bool,
    light_selected: bool,
    selected_object: Option<u32>
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
                    scene_layout: &SceneLayout,
    ) {
        self.gui.begin_frame();
        let context = self.gui.context();
        Self::set_font_sizes(&context);

        let mut ui = Ui::new(context.clone(), "ui".into(), UiBuilder::new());

        // egui::Panel::top("topPanel").show_inside(&mut ui, |ui| {
        //     egui::Button::selectable(true, "fileSelect").
        // });

        egui::Panel::left("sidePanel")
            .resizable(false)
            .show_inside(&mut ui, |ui| {
                ui.add_space(4.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Object tree");
                });
                ui.separator();
                self.walk_through_tree(scene_layout, &scene_layout.scene_tree, &context, ui);
            });

    }

    fn walk_through_tree(&mut self,
                         scene_layout: &SceneLayout, scene_tree: &SceneTree,
                         context: &Context, ui: &mut Ui)
    {
        let cur_object = scene_layout.scene_objects.get(scene_tree.object_id);
        let header_name = cur_object.name.clone() + "_header";
        let button_name = cur_object.name.clone() + "_button";

        let show_item_label = |ui: &mut Ui| {
            let object_selected = self.state.selected_object.is_some_and(|id| {
                id == cur_object.id
            });
            if ui.selectable_label(object_selected, cur_object.name.clone()).clicked() {
                info!("{}", cur_object.name)
            }
        };

        if scene_tree.children.is_empty() {
            show_item_label(ui);
        } else {
            egui::collapsing_header::CollapsingState::load_with_default_open(&context, header_name.into(), false)
                .show_header(ui, |ui| {
                    show_item_label(ui);
                })
                .body(|ui| {
                    for child_tree in scene_tree.children.iter() {
                        self.walk_through_tree(scene_layout, child_tree, context, ui);
                    }
                });
        }
    }

}