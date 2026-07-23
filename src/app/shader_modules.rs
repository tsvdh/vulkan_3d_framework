pub mod vs_mod_shadow {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/shadow.vert",
        custom_derives: [Default, Copy, Clone],
        define: [("edit_id", "91aaxxd3-5aac-4cb9-9818-84b88x4124b8")]
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
        define: [("edit_id", "x45x777x-x8c5-48xd-b563-cc")]
    }
}

pub mod fs_mod_render {
    use crate::app::ui::{vec3_drag_values_int_range, ControlUi};
    use egui::{DragValue, Ui};
    use serde::Deserialize;

    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/render.frag",
        custom_derives: [Default, Copy, Clone, Deserialize],
        define: [("edit_id", "5c921c32-4337-416d-889a-d55b84x5d3dx")]
    }

    impl ControlUi for PhongComponent {
        fn control_ui(&mut self, ui: &mut Ui) {
            ui.horizontal(|ui| {
                vec3_drag_values_int_range(ui, &mut self.color, 1.0, 0, 255);
                ui.add_space(10.0);
                ui.label("coef.: ");
                ui.add(DragValue::new(&mut self.coefficient).speed(0.01).range(0..=1));
            });
        }
    }
    impl ControlUi for PhongMaterial {
        fn control_ui(&mut self, ui: &mut Ui) {
            ui.label("Ambient");
            self.ambient.control_ui(ui);
            ui.add_space(8.0);
            ui.label("Diffuse");
            self.diffuse.control_ui(ui);
            ui.add_space(8.0);
            ui.label("Specular");
            self.specular.control_ui(ui);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("Shininess: ");
                ui.add(DragValue::new(&mut self.shininess).speed(0.1).range(0..=1024));
            });
        }
    }
}