mod app;
pub mod scripts;

use glam::{Mat4, Vec3};
use crate::app::App;

fn main() {
    // println!("{:}", Mat4::look_at_lh(Vec3::new(0.0, 0.0, -3.0), Vec3::ZERO, Vec3::Y));
    // println!("{:}", Mat4::look_to_lh(Vec3::ZERO, Vec3::Z, Vec3::Y) * (Mat4::from_translation(Vec3::new(0.0, 0.0, -3.0))).inverse());
    //
    // println!("{:}", Mat4::from_rotation_x(10.0));
    // println!("{:}", Mat4::from_rotation_x(-10.0).inverse());

    App::start();
}