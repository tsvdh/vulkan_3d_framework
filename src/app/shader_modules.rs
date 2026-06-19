pub mod vertex_shader_module {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/shader.vert",
        custom_derives: [Default, Copy, Clone],
        define: [("edit_id", "4xx2ax83b-4xd4-43a1-a3a2-9d88ax8424x7")]
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