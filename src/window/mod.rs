use std::rc::Rc;

use glow::Context;

pub mod glfw_window;

pub trait OverlayWindow {
    fn create(
        floating: bool,
        transparent: bool,
        passthrough: bool,
        decorated: bool,
    ) -> anyhow::Result<Self>
    where
        Self: Sized;

    fn set_inner_size(&self, width: i32, height: i32);

    fn set_inner_position(&self, xpos: i32, ypos: i32);

    // pub fn decorations(&self, decorated: bool) {
    //     self.window.borrow_mut().set_decorated(decorated);
    // }
    // pub fn _input_passthrough(&self) {
    //     // self.window.borrow_mut().set
    // }
    // pub fn _transparent(&self) {

    // }

    fn get_inner_size(&self) -> (i32, i32);
    fn get_inner_position(&self) -> (i32, i32);

    fn redraw_request(&self);

    fn should_close(&self) -> bool;
    fn get_gl_context(&self) -> Rc<Context>;
}
