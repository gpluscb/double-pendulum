use crate::DoublePendulumCollection;

pub mod image;
pub mod sdl2;

pub trait Renderer {
    fn render_frame(&mut self, pendulums: &DoublePendulumCollection) -> Result<(), String>;
}
