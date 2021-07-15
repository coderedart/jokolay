use std::{cell::RefCell, rc::Rc, sync::mpsc::Receiver};

use anyhow::Context as _;

use egui::{Event, RawInput};
use egui::{Pos2, Rect};
use glfw::{Glfw, Window, WindowEvent};
use glow::{Context, HasContext};
use nalgebra_glm::IVec2;

use crate::input::{glfw_to_egui_key, GlobalInputState};

pub struct GlfwWindow {
    pub global_input_state: Rc<RefCell<GlobalInputState>>,
    pub glfw_events: Rc<Receiver<(f64, WindowEvent)>>,
    pub gl: Rc<glow::Context>,
    pub window: Rc<RefCell<Window>>,
    pub glfw: Rc<RefCell<Glfw>>,
    pub window_pos: (i32, i32),
    pub window_size: (i32, i32),
}
impl GlfwWindow {
    pub fn create(
        floating: bool,
        transparent: bool,
        passthrough: bool,
        decorated: bool,
    ) -> anyhow::Result<GlfwWindow> {
        let mut glfw =
            glfw::init(glfw::FAIL_ON_ERRORS).context("failed to initialize glfw window")?;
        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));

        glfw.window_hint(glfw::WindowHint::Floating(floating));

        glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(transparent));

        glfw.window_hint(glfw::WindowHint::MousePassthrough(passthrough));

        glfw.window_hint(glfw::WindowHint::Decorated(decorated));

        glfw.window_hint(glfw::WindowHint::DoubleBuffer(false));

        let (mut window, events) = glfw
            .create_window(800, 600, "Jokolay", glfw::WindowMode::Windowed)
            .context("Failed to create GLFW window")?;

        window.set_key_polling(true);
        glfw::Context::make_current(&mut window);
        window.set_framebuffer_size_polling(true);
        window.set_close_polling(true);
        window.set_pos_polling(true);
        window.set_key_polling(true);
        window.set_mouse_button_polling(true);
        window.set_scroll_polling(true);
        let gl = unsafe {
            glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _)
        };
        let window = Rc::new(RefCell::new(window));
        let global_input_state = Rc::new(RefCell::new(GlobalInputState::new()));
        let (xpos, ypos) = window.borrow().get_pos();
        let (width, height) = window.borrow().get_framebuffer_size();
        Ok(GlfwWindow {
            glfw: Rc::new(RefCell::new(glfw)),
            window,
            gl: Rc::new(gl),
            glfw_events: Rc::new(events),
            global_input_state,
            window_pos: (xpos, ypos),
            window_size: (width, height),
        })
    }

    pub fn set_inner_size(&self, width: i32, height: i32) {
        self.window.borrow_mut().set_size(width, height);
    }

    pub fn set_inner_position(&self, xpos: i32, ypos: i32) {
        self.window.borrow_mut().set_pos(xpos, ypos);
    }

    // pub fn decorations(&self, decorated: bool) {
    //     self.window.borrow_mut().set_decorated(decorated);
    // }
    // pub fn _input_passthrough(&self) {
    //     // self.window.borrow_mut().set
    // }
    // pub fn _transparent(&self) {

    // }

    pub fn get_inner_size(&self) -> (i32, i32) {
        self.window.borrow_mut().get_framebuffer_size()
    }

    pub fn get_inner_position(&self) -> (i32, i32) {
        self.window.borrow_mut().get_pos()
    }

    pub fn redraw_request(&self) {
        // self.window.borrow_mut().swap_buffers();
        unsafe { self.gl.flush() };
    }

    pub fn should_close(&self) -> bool {
        self.window.borrow().should_close()
    }

    pub fn get_gl_context(&self) -> Rc<Context> {
        self.gl.clone()
    }
}

impl GlfwWindow {
    // pub fn send_events_to_imgui(&mut self, io: &mut Io) {
    //     self.glfw.borrow_mut().poll_events();
    //     for (_, event) in glfw::flush_messages(&self.glfw_events) {
    //         gui::iapp::iglfw::handle_event(io, &event);
    //     }
    // }
    pub fn process_events(&mut self, input: &mut RawInput) {
        self.glfw.borrow_mut().poll_events();

        let (xpos, ypos) = self.window_pos;
        let (_width, _height) = self.window_size;
        let mouse = self.global_input_state.borrow().dq.query_pointer();
        let egui_mouse_position = Pos2::new(
            (mouse.coords.0 - xpos) as f32,
            (mouse.coords.1 - ypos) as f32,
        );
        {
            let mut input_state = self.global_input_state.borrow_mut();
            if input_state.global_mouse_position[0] != mouse.coords.0
                || input_state.global_mouse_position[1] != mouse.coords.1
            {
                input.events.push(Event::PointerMoved(egui_mouse_position));
                input_state.global_mouse_position = IVec2::new(mouse.coords.0, mouse.coords.1);
            }
        }
        for (_, event) in glfw::flush_messages(&self.glfw_events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    // make sure the viewport matches the new window dimensions; note that width and
                    // height will be significantly larger than specified on retina displays.
                    unsafe {
                        self.gl.viewport(0, 0, width, height);
                    }
                    input.screen_rect = Some(Rect::from_two_pos(
                        Pos2::default(),
                        Pos2::new(width as f32, height as f32),
                    ));
                }
                WindowEvent::Pos(x, y) => {
                    self.window_pos = (x, y);
                }
                WindowEvent::Close => {
                    println!("closing");
                    self.window.borrow_mut().set_should_close(true);
                }
                WindowEvent::MouseButton(button, action, modifiers) => {
                    let ebutton = match button {
                        glfw::MouseButton::Button1 => egui::PointerButton::Primary,
                        glfw::MouseButton::Button2 => egui::PointerButton::Secondary,
                        glfw::MouseButton::Button3 => egui::PointerButton::Middle,
                        // glfw::MouseButton::Button4 => todo!(),
                        // glfw::MouseButton::Button5 => todo!(),
                        // glfw::MouseButton::Button6 => todo!(),
                        // glfw::MouseButton::Button7 => todo!(),
                        // glfw::MouseButton::Button8 => todo!(),
                        _ => egui::PointerButton::Primary,
                    };
                    let epress = match action {
                        glfw::Action::Release => false,
                        glfw::Action::Press => true,
                        _ => panic!("glfw mouse repeat {} {}", file!(), line!()),
                    };
                    let emodifiers = egui::Modifiers {
                        alt: modifiers.contains(glfw::Modifiers::Alt),
                        ctrl: modifiers.contains(glfw::Modifiers::Control),
                        shift: modifiers.contains(glfw::Modifiers::Shift),
                        mac_cmd: false,
                        command: modifiers.contains(glfw::Modifiers::Control),
                    };
                    input.events.push(egui::Event::PointerButton {
                        pos: egui_mouse_position,
                        button: ebutton,
                        pressed: epress,
                        modifiers: emodifiers,
                    });
                }

                WindowEvent::Scroll(x, y) => {
                    input.scroll_delta = egui::Vec2::new((x * 10.0) as f32, (y * 10.0) as f32);
                }
                WindowEvent::Key(key, _, action, modifiers) => {
                    let ekey = glfw_to_egui_key(key);
                    if let Some(k) = ekey {
                        let epress = match action {
                            glfw::Action::Release => false,
                            glfw::Action::Press => true,
                            _ => panic!("glfw mouse repeat {} {}", file!(), line!()),
                        };
                        let emodifiers = egui::Modifiers {
                            alt: modifiers.contains(glfw::Modifiers::Alt),
                            ctrl: modifiers.contains(glfw::Modifiers::Control),
                            shift: modifiers.contains(glfw::Modifiers::Shift),
                            mac_cmd: false,
                            command: modifiers.contains(glfw::Modifiers::Control),
                        };
                        input.events.push(Event::Key {
                            key: k,
                            pressed: epress,
                            modifiers: emodifiers,
                        });
                    }
                }
                // WindowEvent::Size(_, _) => todo!(),
                // WindowEvent::Refresh => todo!(),
                // WindowEvent::Focus(_) => todo!(),
                // WindowEvent::Iconify(_) => todo!(),
                // WindowEvent::FramebufferSize(_, _) => todo!(),
                // WindowEvent::CursorPos(_, _) => todo!(),
                // WindowEvent::CursorEnter(_) => todo!(),
                // WindowEvent::Char(_) => todo!(),
                // WindowEvent::CharModifiers(_, _) => todo!(),
                // WindowEvent::FileDrop(_) => todo!(),
                // WindowEvent::Maximize(_) => todo!(),
                // WindowEvent::ContentScale(_, _) => todo!(),
                _ => unimplemented!(),
            }
        }
    }
}
