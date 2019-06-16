#[cfg(not(target_os = "windows"))]
use std::os::unix::ffi::OsStrExt;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
use std::{
    ffi::{CStr, CString},
    path::Path,
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

pub struct WKSettingsBuilder(ffi::wksettings_t);
pub struct WKSettings(ffi::wksettings_t);

impl WKSettingsBuilder {
    fn new() -> Self {
        WKSettingsBuilder(unsafe { ffi::wksettings_create() })
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

    pub fn build(mut self) -> WKSettings {
        assert!(!self.0.is_null());
        let inst = WKSettings(self.0);
        self.0 = ptr::null_mut();
        inst
    }
}

impl Drop for WKSettingsBuilder {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                ffi::wksettings_free(self.0);
            }
            self.0 = ptr::null_mut();
        }
    }
}

impl WKSettings {
    pub fn builder() -> WKSettingsBuilder {
        WKSettingsBuilder::new()
    }
}

impl Drop for WKSettings {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                ffi::wksettings_free(self.0);
            }
            self.0 = ptr::null_mut();
        }
    }
}

pub struct WKConverter {
    ptr: ffi::wkconverter_t,
    settings: WKSettings,
}

extern "C" fn on_log(_: *mut std::os::raw::c_void, msg: *const std::os::raw::c_char) {
    let cstr = unsafe { CStr::from_ptr(msg) };
    let s = cstr.to_str().expect("log message not utf8");
    println!("log: {}", s);
}

extern "C" fn on_error(_: *mut std::os::raw::c_void, msg: *const std::os::raw::c_char) {
    let cstr = unsafe { CStr::from_ptr(msg) };
    let s = cstr.to_str().expect("error message not utf8");
    eprintln!("error: {}", s);
}

impl WKConverter {
    pub fn new(settings: WKSettings) -> Self {
        Self {
            ptr: unsafe { ffi::wkconverter_create(settings.0) },
            settings,
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        unsafe {
            ffi::wkconverter_on_log(self.ptr, Some(on_log));
            ffi::wkconverter_on_error(self.ptr, Some(on_error));
        };

        let res = unsafe { ffi::wkconverter_run(self.ptr) };

        if res != 0 {
            Err("unk".to_string())
        } else {
            Ok(())
        }
    }
}

impl Drop for WKConverter {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ffi::wkconverter_free(self.ptr);
            }
            self.ptr = ptr::null_mut();
        }
    }
}
