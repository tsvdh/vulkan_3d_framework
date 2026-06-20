pub mod vertex_shader_module {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/shader.vert",
        custom_derives: [Default, Copy, Clone],
        define: [("edit_id", "x45x777x-x8c5-48xd-b563-32x68cx3526d")]
    }
}

pub mod fragment_shader_module {
    use serde::Deserialize;

    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/shader.frag",
        custom_derives: [Default, Copy, Clone, Deserialize],
        define: [("edit_id", "438adca3-5x8d-47bx-9938-8dxx57x2bx52")]
    }
}