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

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// TODO make libwk accept wchar_t on windows, then use std::os::windows::ffi::OsStrExt to convert
// to wchar_t
#[cfg(not(target_os = "windows"))]
fn path_to_cstring(path: &Path) -> CString {
    CString::new(path.as_os_str().as_bytes()).expect("invalid path")
}

#[derive(Debug, Clone, Copy)]
enum IndexType {
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

    pub fn copy_maps(mut self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_copy_maps(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    pub fn copy_custom_maps(mut self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_copy_custom_maps(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    pub fn restricted_civ_mods(mut self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_restricted_civ_mods(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    pub fn fix_flags(mut self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_fix_flags(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    pub fn replace_tooltips(mut self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_replace_tooltips(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    pub fn use_grid(mut self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_use_grid(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    pub fn use_short_walls(mut self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_use_short_walls(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    pub fn language(mut self, code: &str) -> Self {
        let cstr = CString::new(code).expect("invalid language");
        unsafe { ffi::wksettings_language(self.0, cstr.as_ptr() as *const i8) };
        self
    }

    pub fn patch(mut self, patch: i32) -> Self {
        unsafe { ffi::wksettings_patch(self.0, patch) };
        self
    }

    pub fn hotkeys(mut self, choice: i32) -> Self {
        unsafe { ffi::wksettings_hotkeys(self.0, choice) };
        self
    }

    pub fn dlc_level(mut self, level: i32) -> Self {
        unsafe { ffi::wksettings_dlc_level(self.0, level) };
        self
    }

    pub fn resource_path(mut self, path: &Path) -> Self {
        let cstr = path_to_cstring(path);
        unsafe { ffi::wksettings_resource_path(self.0, cstr.as_ptr() as *const i8) };
        self
    }

    pub fn hd_path(mut self, path: &Path) -> Self {
        let cstr = path_to_cstring(path);
        unsafe { ffi::wksettings_hd_path(self.0, cstr.as_ptr() as *const i8) };
        self
    }

    pub fn output_path(mut self, path: &Path) -> Self {
        let cstr = path_to_cstring(path);
        unsafe { ffi::wksettings_output_path(self.0, cstr.as_ptr() as *const i8) };
        self
    }

    pub fn voobly_path(mut self, path: &Path) -> Self {
        let cstr = path_to_cstring(path);
        unsafe { ffi::wksettings_voobly_path(self.0, cstr.as_ptr() as *const i8) };
        self
    }

    pub fn up_path(mut self, path: &Path) -> Self {
        let cstr = path_to_cstring(path);
        unsafe { ffi::wksettings_up_path(self.0, cstr.as_ptr() as *const i8) };
        self
    }

    pub fn mod_name(mut self, name: &str) -> Self {
        let cstr = CString::new(name).expect("invalid mod name");
        unsafe { ffi::wksettings_mod_name(self.0, cstr.as_ptr() as *const i8) };
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

#[derive(Default)]
pub struct ConvertListener {
    on_log: Option<Box<dyn Fn(&str) -> ()>>,
}

impl ConvertListener {
    pub fn on_log(&mut self, callback: impl Fn(&str) -> () + 'static) {
        self.on_log = Some(Box::new(callback));
    }

    fn log(&self, text: &str) {
        if let Some(on_log) = &self.on_log {
            on_log(text);
        }
    }
}

struct ConvertContext {
    last_error: Option<String>,
    listener: ConvertListener,
    _pin: PhantomPinned,
}

impl ConvertContext {
    pub fn new(listener: ConvertListener) -> Pin<Box<Self>> {
        Box::pin(ConvertContext {
            last_error: Default::default(),
            listener,
            _pin: PhantomPinned,
        })
    }

    unsafe fn from_ptr<'a>(ptr: *mut std::os::raw::c_void) -> &'a mut Self {
        (ptr as *mut ConvertContext).as_mut().expect("ConvertContext was null; this is a bug")
    }
}

pub struct Converter {
    ptr: ffi::wkconverter_t,
    context: Pin<Box<ConvertContext>>,
    settings: ConvertOptions,
}

extern "C" fn on_log(ctx: *mut std::os::raw::c_void, msg: *const std::os::raw::c_char) {
    let ctx = unsafe { ConvertContext::from_ptr(ctx) };
    let cstr = unsafe { CStr::from_ptr(msg) };
    let s = cstr.to_str().expect("log message not utf8");
    ctx.listener.log(s);
}

extern "C" fn on_error(ctx: *mut std::os::raw::c_void, msg: *const std::os::raw::c_char) {
    let ctx = unsafe { ConvertContext::from_ptr(ctx) };
    let cstr = unsafe { CStr::from_ptr(msg) };
    let s = cstr.to_str().expect("error message not utf8");
    ctx.last_error = Some(s.to_string());
}

impl Converter {
    pub fn new(settings: ConvertOptions, listener: ConvertListener) -> Self {
        let mut context = ConvertContext::new(listener);

        Self {
            ptr: unsafe {
                let mut_ref = Pin::as_mut(&mut context);
                let context_ptr = Pin::get_unchecked_mut(mut_ref)
                    as *mut ConvertContext
                    as *mut std::os::raw::c_void;
                ffi::wkconverter_create(settings.0, context_ptr)
            },
            context,
            settings,
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        unsafe {
            ffi::wkconverter_on_log(self.ptr, Some(on_log));
            ffi::wkconverter_on_error(self.ptr, Some(on_error));
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
