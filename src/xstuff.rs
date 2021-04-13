use std::{ffi::CString, ptr};
use x11::xlib::{Display, Window, XDefaultScreen, XDefaultScreenOfDisplay, XFlush, XOpenDisplay, XRootWindowOfScreen, XStoreName};

pub struct WindowSystem {
    display: *mut Display,
    root: Window,
}

impl WindowSystem {
    pub fn new() -> WindowSystem {
        unsafe {
            let display = XOpenDisplay(ptr::null_mut());
            let screen = XDefaultScreenOfDisplay(display);
            let root = XRootWindowOfScreen(screen);
            WindowSystem { display, root }
        }
    }
    pub fn draw(&self, name: String) {
        unsafe {
            let c_str = CString::new(name).unwrap();
            XStoreName(self.display, self.root, c_str.as_ptr() as *const i8);
            XFlush(self.display);
        }
    }
}
