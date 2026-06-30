pub mod vertex_shader_module {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/shader.vert",
        custom_derives: [Default, Copy, Clone],
        define: [("edit_id", "x45x777x-x8c5-48xd-b563-32x68cx3526d")]
    }
}

pub mod fragment_shader_module {
    use crate::app::ui::{vec3_drag_values_int_range, ControlUi};
    use egui::{DragValue, Ui};
    use serde::Deserialize;

    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/shader.frag",
        custom_derives: [Default, Copy, Clone, Deserialize],
        define: [("edit_id", "438adca3-5x8d-47bx-9938-8dxx52bfx52")]
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