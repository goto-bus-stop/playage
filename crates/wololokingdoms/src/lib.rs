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
    #[allow(dead_code)]
    fn to_ffi(self) -> ffi::WKIndexType {
        match self {
            IndexType::IndexOnly => ffi::WKIndexType_DRSIndexOnly,
            IndexType::Expansion => ffi::WKIndexType_DRSExpansionResources,
            IndexType::Terrain => ffi::WKIndexType_DRSTerrainResources,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HotkeyStyle {
    /// Use AoC hotkeys in WololoKingdoms.
    AoC = 1,
    /// Use HD Edition hotkeys in WololoKingdoms.
    HD = 2,
    /// Use HD Edition hotkeys in WololoKingdoms *and* the base game.
    HDForBoth = 3,
}

/// The DLCs that should be converted.
///
/// These stack linearly, you cannot convert African Kingdoms if you do not own The Forgotten.
#[derive(Debug, Clone, Copy)]
pub enum DLCLevel {
    /// Only convert the base game.
    Conquerors = 0,
    /// Convert the base game and the Forgotten expansion.
    Forgotten = 1,
    /// Convert the base game and the Forgotten and African Kingdoms expansions.
    AfricanKingdoms = 2,
    /// Convert the base game and the Forgotten, African Kingdoms, and Rise of the Rajas expansions.
    RiseOfTheRajas = 3,
}

/// Builder struct to create options for the WololoKingdoms converter.
pub struct ConvertOptionsBuilder(ffi::wksettings_t);
/// Struct holding a reference to completed convert options.
pub struct ConvertOptions(ffi::wksettings_t);

impl ConvertOptionsBuilder {
    fn new() -> Self {
        ConvertOptionsBuilder(unsafe { ffi::wksettings_create() })
    }

    /// Should the HD Edition builtin map scripts be converted?
    pub fn copy_maps(self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_copy_maps(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    /// Should any user-installed map scripts from the Steam Workshop be converted?
    pub fn copy_custom_maps(self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_copy_custom_maps(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    pub fn restricted_civ_mods(self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_restricted_civ_mods(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    /// Should the flag locations on buildings be converted?
    pub fn fix_flags(self, enabled: bool) -> Self {
        unsafe { ffi::wksettings_fix_flags(self.0, if enabled { 1 } else { 0 }) };
        self
    }

    /// Should the tooltip descriptions be extended?
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

    /// Set the language to use.
    pub fn language(self, code: &str) -> Self {
        let cstr = CString::new(code).expect("invalid language");
        unsafe { ffi::wksettings_language(self.0, cstr.as_ptr() as *const i8) };
        self
    }

    /// Set the patch ID to use.
    pub fn patch(self, patch: i32) -> Self {
        unsafe { ffi::wksettings_patch(self.0, patch) };
        self
    }

    /// Set the hotkey style to use.
    pub fn hotkeys(self, choice: HotkeyStyle) -> Self {
        unsafe { ffi::wksettings_hotkeys(self.0, choice as i32) };
        self
    }

    /// Set the DLCs to convert.
    pub fn dlc_level(self, level: DLCLevel) -> Self {
        unsafe { ffi::wksettings_dlc_level(self.0, level as i32) };
        self
    }

    /// Set the path where the WololoKingdoms converter can find the resource files it needs for conversion.
    pub fn resource_path(self, path: &Path) -> Self {
        let cstr = path_to_cpath(path);
        unsafe { ffi::wksettings_resource_path(self.0, cstr.as_ptr() as *const ffi::path_char_t) };
        self
    }

    /// Set the path where the HD Edition is installed.
    pub fn hd_path(self, path: &Path) -> Self {
        let cstr = path_to_cpath(path);
        unsafe { ffi::wksettings_hd_path(self.0, cstr.as_ptr() as *const ffi::path_char_t) };
        self
    }

    /// Set the path where the Conquerors is installed.
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

    /// Set the name of the mod (default "WK").
    pub fn mod_name(self, name: &str) -> Self {
        let cstr = CString::new(name).expect("invalid mod name");
        unsafe { ffi::wksettings_mod_name(self.0, cstr.as_ptr() as *const ffi::path_char_t) };
        self
    }

    /// Consume the builder into a ConvertOptions struct.
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
    /// Create an options builder.
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

/// Callbacks for the installation process.
pub trait ConvertListener {
    fn log(&mut self, _text: &str) {}
    fn set_info(&mut self, _text: &str) {}
    fn progress(&mut self, _progress: f32) {}
    fn finished(&mut self) {}
}

/// Wrap up a ConvertListener implementation to convert the WK converter callbacks into something
/// more rusty.
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

/// Context argument for libwololokingdoms callbacks.
///
/// Since we will be giving the WK converter library a C pointer to this structure, it cannot be moved.
struct ConvertContext {
    last_error: Option<String>,
    listener: Listener,
}

impl ConvertContext {
    /// Create a context argument with some callbacks.
    fn new(listener: Listener) -> Box<Self> {
        Box::new(ConvertContext {
            last_error: Default::default(),
            listener,
        })
    }

    /// Reify a context instance from the context argument inside a WK converter callback.
    unsafe fn from_ptr<'a>(ptr: *mut std::os::raw::c_void) -> &'a mut Self {
        (ptr as *mut ConvertContext)
            .as_mut()
            .expect("ConvertContext was null; this is a bug")
    }
}

pub struct Converter {
    /// The underlying C++ WK converter.
    ptr: ffi::wkconverter_t,
    /// Context argument for the WK converter callbacks.
    context: Box<ConvertContext>,
    /// Keep this around while the converter lives, so it isn't dropped too early.
    _settings: ConvertOptions,
}

/// WK converter `log` callback.
extern "C" fn on_log(ctx: *mut std::os::raw::c_void, msg: *const std::os::raw::c_char) {
    let ctx = unsafe { ConvertContext::from_ptr(ctx) };
    let cstr = unsafe { CStr::from_ptr(msg) };
    let s = cstr.to_str().expect("log message not utf8");
    ctx.listener.log(s);
}

/// WK converter `setInfo` callback.
extern "C" fn on_set_info(ctx: *mut std::os::raw::c_void, msg: *const std::os::raw::c_char) {
    let ctx = unsafe { ConvertContext::from_ptr(ctx) };
    let cstr = unsafe { CStr::from_ptr(msg) };
    let s = cstr.to_str().expect("info message not utf8");
    ctx.listener.set_info(s);
}

/// WK converter `progress` callback.
extern "C" fn on_progress(ctx: *mut std::os::raw::c_void, percent: i32) {
    let ctx = unsafe { ConvertContext::from_ptr(ctx) };
    ctx.listener.progress(percent);
}

/// WK converter `error` callback.
extern "C" fn on_error(ctx: *mut std::os::raw::c_void, msg: *const std::os::raw::c_char) {
    let ctx = unsafe { ConvertContext::from_ptr(ctx) };
    let cstr = unsafe { CStr::from_ptr(msg) };
    let s = cstr.to_str().expect("error message not utf8");
    ctx.last_error = Some(s.to_string());
}

/// WK converter `finished` callback.
extern "C" fn on_finished(ctx: *mut std::os::raw::c_void) {
    let ctx = unsafe { ConvertContext::from_ptr(ctx) };
    ctx.listener.finish();
}

impl Converter {
    /// Create a converter with the given options and callbacks.
    pub fn new(settings: ConvertOptions, listener: Box<dyn ConvertListener>) -> Self {
        let listener = Listener::new(listener);
        let mut context = ConvertContext::new(listener);

        Self {
            ptr: unsafe {
                let context_ptr =
                    context.as_mut() as *mut ConvertContext as *mut std::os::raw::c_void;
                ffi::wkconverter_create(settings.0, context_ptr)
            },
            context,
            _settings: settings,
        }
    }

    fn install_callbacks(&mut self) {
        unsafe {
            ffi::wkconverter_on_log(self.ptr, Some(on_log));
            ffi::wkconverter_on_set_info(self.ptr, Some(on_set_info));
            ffi::wkconverter_on_progress(self.ptr, Some(on_progress));
            ffi::wkconverter_on_error(self.ptr, Some(on_error));
            ffi::wkconverter_on_finished(self.ptr, Some(on_finished));
        };
    }

    /// Run the conversion!
    pub fn run(mut self) -> Result<(), String> {
        self.install_callbacks();

        let res = unsafe { ffi::wkconverter_run(self.ptr) };

        if res != 0 {
            let err = self.context.last_error.take()
                .unwrap_or_else(|| "Unknown error".to_string());
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
