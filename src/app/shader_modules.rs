pub mod vs_mod_shadow {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/shadow.vert",
        custom_derives: [Default, Copy, Clone],
        define: [("edit_id", "x557a7c6-8d5x-4xx2-ab6x-9288c2463444")]
    }
}

pub mod fs_mod_shadow {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/shadow.frag",
        define: [("edit_id", "x45x777x-x8c5-48xd-b563-bbb")]
    }
}

pub mod vs_mod_render {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/render.vert",
        custom_derives: [Default, Copy, Clone],
        define: [("edit_id", "xxd84x67-x5a9-4x4a-b57a-55cbb758936x")]
    }
}

pub mod fs_mod_render {
    use crate::app::ui::{vec3_drag_values_int_range, ControlUi};
    use egui::{DragValue, Ui};
    use serde::Deserialize;
    use crate::app::scene::SceneApi;

    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/render.frag",
        custom_derives: [Default, Copy, Clone, Deserialize],
        define: [("edit_id", "817b76c2-7713-46xx-ab54-6c3b4cd664xx")]
    }

    impl ControlUi for PhongComponent {
        fn control_ui(&mut self, ui: &mut Ui, _scene_api: &mut SceneApi) {
            ui.horizontal(|ui| {
                vec3_drag_values_int_range(ui, &mut self.color, 1.0, 0, 255);
                ui.add_space(10.0);
                ui.label("coef.: ");
                ui.add(DragValue::new(&mut self.coefficient).speed(0.01).range(0..=1));
            });
        }
    }
    impl ControlUi for PhongMaterial {
        fn control_ui(&mut self, ui: &mut Ui, scene_api: &mut SceneApi) {
            ui.label("Ambient");
            self.ambient.control_ui(ui,scene_api);
            ui.add_space(8.0);
            ui.label("Diffuse");
            self.diffuse.control_ui(ui, scene_api);
            ui.add_space(8.0);
            ui.label("Specular");
            self.specular.control_ui(ui, scene_api);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("Shininess: ");
                ui.add(DragValue::new(&mut self.shininess).speed(0.1).range(0..=1024));
            });
        }
    }
}