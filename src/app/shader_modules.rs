pub mod vertex_shader_module {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/shader.vert",
        custom_derives: [Default, Copy, Clone],
        define: [("edit_id", "x3axdx7x-xcxx-4axa-aaex-833993bdx87d")]
    }
}

pub mod fragment_shader_module {
    use serde::Deserialize;

    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/shader.frag",
        custom_derives: [Default, Copy, Clone, Deserialize],
        define: [("edit_id", "8bx5xcxa-xxca-4286-aa11-ccxb4cc28xcc")]
    }
}