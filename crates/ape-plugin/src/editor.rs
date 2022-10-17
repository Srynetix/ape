use std::sync::Arc;

use crate::Parameters;
use baseview::{Size, WindowHandle, WindowOpenOptions, WindowScalePolicy};
use egui::Context;
use egui_baseview::EguiWindow;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use vst::{editor::Editor, prelude::PluginParameters};

const WINDOW_WIDTH: usize = 256;
const WINDOW_HEIGHT: usize = 256;

pub struct PluginEditor {
    pub params: Arc<Parameters>,
    pub window_handle: Option<WindowParent>,
    pub is_open: bool,
}

impl Editor for PluginEditor {
    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn size(&self) -> (i32, i32) {
        (WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
    }

    fn is_open(&mut self) -> bool {
        self.is_open
    }

    fn close(&mut self) {
        self.is_open = false;
        if let Some(mut handle) = self.window_handle.take() {
            handle.0.close();
        }
    }

    fn open(&mut self, parent: *mut std::os::raw::c_void) -> bool {
        log::info!("Editor open");
        match self.is_open {
            true => false,
            false => {
                self.is_open = true;
                let settings = WindowOpenOptions {
                    title: "SynthTest".into(),
                    size: Size::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64),
                    scale: WindowScalePolicy::SystemScaleFactor,
                    gl_config: None,
                };

                let window_handle = EguiWindow::open_parented(
                    &VstParent(parent),
                    settings,
                    self.params.clone(),
                    |_ctx, _queue, _state| {},
                    |ctx, _, state| {
                        draw_ui(ctx, state);
                    },
                );

                self.window_handle = Some(WindowParent(window_handle));
                true
            }
        }
    }
}

#[inline(always)]
fn draw_ui(ctx: &Context, params: &mut Arc<Parameters>) -> egui::Response {
    egui::CentralPanel::default()
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("hello rust");
                ui.label(format!(
                    "Modulation: {}",
                    params.get_parameter(crate::Parameter::Modulation as i32)
                ));

                let mut val = params.modulation.get();
                if ui
                    .add(egui::Slider::new(&mut val, 0f32..=10f32).text("Modulation"))
                    .changed()
                {
                    params.modulation.set(val);
                }
            })
        })
        .response
}

struct VstParent(*mut ::std::ffi::c_void);
unsafe impl Send for VstParent {}

pub struct WindowParent(pub WindowHandle);
unsafe impl Send for WindowParent {}

unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::Win32Handle;

        let mut hndl = Win32Handle::empty();
        hndl.hwnd = self.0;

        RawWindowHandle::Win32(hndl)
    }
}
