#[cfg(not(target_os = "windows"))]
use std::os::unix::ffi::OsStrExt;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
use std::{
    ffi::{CStr, CString},
    marker::PhantomPinned,
    path::Path,
    pin::Pin,
    ptr,
};

mod ffi {
    #![allow(non_camel_case_types)]
    #![allow(non_upper_case_globals)]
    #![allow(unused)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(not(target_os = "windows"))]
fn path_to_cpath(path: &Path) -> CString {
    CString::new(path.as_os_str().as_bytes()).expect("invalid path")
}

#[cfg(target_os = "windows")]
fn path_to_cpath(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().collect()
}

#[derive(Debug, Clone, Copy)]
pub enum IndexType {
    IndexOnly,
    Expansion,
    Terrain,
}

impl IndexType {
    fn to_ffi(self) -> ffi::WKIndexType {
        match self {
            IndexType::IndexOnly => ffi::WKIndexType_DRSIndexOnly,
            IndexType::Expansion => ffi::WKIndexType_DRSExpansionResources,
            IndexType::Terrain => ffi::WKIndexType_DRSTerrainResources,
        }
    }
}

pub struct ConvertOptionsBuilder(ffi::wksettings_t);
pub struct ConvertOptions(ffi::wksettings_t);

impl ConvertOptionsBuilder {
    fn new() -> Self {
        ConvertOptionsBuilder(unsafe { ffi::wksettings_create() })
    }

    pub fn copy_maps(self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_copy_maps(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    pub fn copy_custom_maps(self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_copy_custom_maps(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    pub fn restricted_civ_mods(self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_restricted_civ_mods(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    pub fn fix_flags(self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_fix_flags(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    pub fn replace_tooltips(self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_replace_tooltips(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    /// Install grid terrain textures.
    pub fn use_grid(self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_use_grid(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    /// Install the short walls mod.
    pub fn use_short_walls(self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_use_short_walls(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    pub fn language(self, code: &str) -> Self {
        let cstr = CString::new(code).expect("invalid language");
        unsafe { ffi::wksettings_language(self.0, cstr.as_ptr() as *const i8) };
        self
    }

    pub fn patch(self, patch: i32) -> Self {
        unsafe { ffi::wksettings_patch(self.0, patch) };
        self
    }

    pub fn hotkeys(self, choice: i32) -> Self {
        unsafe { ffi::wksettings_hotkeys(self.0, choice) };
        self
    }

    pub fn dlc_level(self, level: i32) -> Self {
        unsafe { ffi::wksettings_dlc_level(self.0, level) };
        self
    }

    pub fn resource_path(self, path: &Path) -> Self {
        let cstr = path_to_cpath(path);
        unsafe { ffi::wksettings_resource_path(self.0, cstr.as_ptr() as *const ffi::path_char_t) };
        self
    }

    pub fn hd_path(self, path: &Path) -> Self {
        let cstr = path_to_cpath(path);
        unsafe { ffi::wksettings_hd_path(self.0, cstr.as_ptr() as *const ffi::path_char_t) };
        self
    }

    pub fn output_path(self, path: &Path) -> Self {
        let cstr = path_to_cpath(path);
        unsafe { ffi::wksettings_output_path(self.0, cstr.as_ptr() as *const ffi::path_char_t) };
        self
    }

    /// Set the Voobly installation path.
    pub fn voobly_path(self, path: &Path) -> Self {
        let cstr = path_to_cpath(path);
        unsafe { ffi::wksettings_voobly_path(self.0, cstr.as_ptr() as *const ffi::path_char_t) };
        self
    }

    /// Set the AoC/UserPatch installation path.
    pub fn up_path(self, path: &Path) -> Self {
        let cstr = path_to_cpath(path);
        unsafe { ffi::wksettings_up_path(self.0, cstr.as_ptr() as *const ffi::path_char_t) };
        self
    }

    pub fn mod_name(self, name: &str) -> Self {
        let cstr = CString::new(name).expect("invalid mod name");
        unsafe { ffi::wksettings_mod_name(self.0, cstr.as_ptr() as *const ffi::path_char_t) };
        self
    }

    pub fn build(mut self) -> ConvertOptions {
        assert!(!self.0.is_null());
        let inst = ConvertOptions(self.0);
        self.0 = ptr::null_mut();
        inst
    }
}

impl Drop for ConvertOptionsBuilder {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                ffi::wksettings_free(self.0);
            }
            self.0 = ptr::null_mut();
        }
    }
}

impl ConvertOptions {
    pub fn builder() -> ConvertOptionsBuilder {
        ConvertOptionsBuilder::new()
    }
}

impl Drop for ConvertOptions {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                ffi::wksettings_free(self.0);
            }
            self.0 = ptr::null_mut();
        }
    }
}

pub trait ConvertListener {
    fn log(&mut self, _text: &str) {}
    fn set_info(&mut self, _text: &str) {}
    fn progress(&mut self, _progress: f32) {}
    fn finished(&mut self) {}
}

pub struct Listener {
    inner: Box<dyn ConvertListener>,
}

impl Listener {
    fn new(inner: Box<dyn ConvertListener>) -> Self {
        Self { inner }
    }

    fn log(&mut self, text: &str) {
        self.inner.log(text);
    }

    fn set_info(&mut self, text: &str) {
        self.inner.set_info(text);
    }

    fn progress(&mut self, percent: i32) {
        self.inner.progress((percent as f32) / 100.0);
    }

    fn finish(&mut self) {
        self.inner.finished()
    }
}

struct ConvertContext {
    last_error: Option<String>,
    listener: Listener,
    _pin: PhantomPinned,
}

impl ConvertContext {
    pub fn new(listener: Listener) -> Pin<Box<Self>> {
        Box::pin(ConvertContext {
            last_error: Default::default(),
            listener,
            _pin: PhantomPinned,
        })
    }

    unsafe fn from_ptr<'a>(ptr: *mut std::os::raw::c_void) -> &'a mut Self {
        (ptr as *mut ConvertContext)
            .as_mut()
            .expect("ConvertContext was null; this is a bug")
    }
}

pub struct Converter {
    ptr: ffi::wkconverter_t,
    context: Pin<Box<ConvertContext>>,
    /// Keep this around while the converter lives, so it isn't dropped too early.
    ///
    /// It does not need to be pinned because it is itself a pointer. The pointed-to data does not
    /// move even if the ConvertOptions object moves.
    _settings: ConvertOptions,
}

extern "C" fn on_log(ctx: *mut std::os::raw::c_void, msg: *const std::os::raw::c_char) {
    let ctx = unsafe { ConvertContext::from_ptr(ctx) };
    let cstr = unsafe { CStr::from_ptr(msg) };
    let s = cstr.to_str().expect("log message not utf8");
    ctx.listener.log(s);
}

extern "C" fn on_set_info(ctx: *mut std::os::raw::c_void, msg: *const std::os::raw::c_char) {
    let ctx = unsafe { ConvertContext::from_ptr(ctx) };
    let cstr = unsafe { CStr::from_ptr(msg) };
    let s = cstr.to_str().expect("info message not utf8");
    ctx.listener.set_info(s);
}

extern "C" fn on_progress(ctx: *mut std::os::raw::c_void, percent: i32) {
    let ctx = unsafe { ConvertContext::from_ptr(ctx) };
    ctx.listener.progress(percent);
}

extern "C" fn on_error(ctx: *mut std::os::raw::c_void, msg: *const std::os::raw::c_char) {
    let ctx = unsafe { ConvertContext::from_ptr(ctx) };
    let cstr = unsafe { CStr::from_ptr(msg) };
    let s = cstr.to_str().expect("error message not utf8");
    ctx.last_error = Some(s.to_string());
}

extern "C" fn on_finished(ctx: *mut std::os::raw::c_void) {
    let ctx = unsafe { ConvertContext::from_ptr(ctx) };
    ctx.listener.finish();
}

impl Converter {
    pub fn new(settings: ConvertOptions, listener: Box<dyn ConvertListener>) -> Self {
        let listener = Listener::new(listener);
        let mut context = ConvertContext::new(listener);

        Self {
            ptr: unsafe {
                let mut_ref = Pin::as_mut(&mut context);
                let context_ptr = Pin::get_unchecked_mut(mut_ref) as *mut ConvertContext
                    as *mut std::os::raw::c_void;
                ffi::wkconverter_create(settings.0, context_ptr)
            },
            context,
            _settings: settings,
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        unsafe {
            ffi::wkconverter_on_log(self.ptr, Some(on_log));
            ffi::wkconverter_on_set_info(self.ptr, Some(on_set_info));
            ffi::wkconverter_on_progress(self.ptr, Some(on_progress));
            ffi::wkconverter_on_error(self.ptr, Some(on_error));
            ffi::wkconverter_on_finished(self.ptr, Some(on_finished));
        };

        let res = unsafe { ffi::wkconverter_run(self.ptr) };
        let context = Pin::as_mut(&mut self.context);
        let context = unsafe { Pin::get_unchecked_mut(context) };

        if res != 0 {
            let err = context.last_error.take().unwrap_or("unk".to_string());
            Err(err)
        } else {
            Ok(())
        }
    }
}

impl Drop for Converter {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ffi::wkconverter_free(self.ptr);
            }
            self.ptr = ptr::null_mut();
        }
    }
}
