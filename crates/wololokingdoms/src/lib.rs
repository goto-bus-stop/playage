use std::path::PathBuf;
use std::sync::Arc;
use std::mem;
use std::ffi::{CStr, CString};
use libc::{c_char, c_void};
#[cfg(not(target_os = "windows"))]
use std::os::unix::ffi::OsStrExt;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

#[repr(C)]
struct FFIWKSettings {
    use_voobly: bool,
    use_exe: bool,
    use_both: bool,
    use_monks: bool,
    use_pw: bool,
    use_walls: bool,
    copy_maps: bool,
    copy_custom_maps: bool,
    restricted_civ_mods: bool,
    use_no_snow: bool,
    fix_flags: bool,
    replace_tooltips: bool,
    use_grid: bool,
    install_directory: *const c_char,
    language: *const c_char,
    dlc_level: i32,
    patch: i32,
    hotkey_choice: i32,
    hd_path: *const c_char,
    out_path: *const c_char,
    voobly_dir: *const c_char,
    up_dir: *const c_char,
    mod_name: *const c_char,
}

#[repr(C)]
struct FFIWKListener {
    data: *mut c_void,
    finished: extern fn(*mut c_void),
    log: extern fn(*mut c_void, *const c_char),
    set_info: extern fn(*mut c_void, *const c_char),
    error: extern fn(*mut c_void, *const c_char),
    create_dialog: extern fn(*mut c_void, *const c_char),
    create_dialog_title: extern fn(*mut c_void, *const c_char, *const c_char),
    create_dialog_replace: extern fn(*mut c_void, *const c_char, *const c_char, *const c_char),
    set_progress: extern fn(*mut c_void, u32),
    install_userpatch: extern fn(*mut c_void, *const c_char, *const *const c_char),
}

extern "C" {
    fn wkconverter_create(settings: *mut FFIWKSettings, listener: *mut FFIWKListener) -> *mut c_void;
    fn wkconverter_run(converter: *mut c_void);
}

#[cfg(target_os = "windows")]
fn encode_path(buf: PathBuf) -> *const c_char {
    buf.as_os_str()
        .encode_wide()
        .collect::<&[u16]>()
        .as_ptr()
}
#[cfg(not(target_os = "windows"))]
fn encode_path(buf: PathBuf) -> *const c_char {
    buf.as_os_str()
        .as_bytes()
        .as_ptr() as *const c_char
}

struct ConvertOptions {
    use_voobly: bool,
    use_exe: bool,
    use_both: bool,
    use_monks: bool,
    use_pw: bool,
    use_walls: bool,
    copy_maps: bool,
    copy_custom_maps: bool,
    restricted_civ_mods: bool,
    use_no_snow: bool,
    fix_flags: bool,
    replace_tooltips: bool,
    use_grid: bool,
    install_directory: PathBuf,
    language: String,
    dlc_level: i32,
    patch: i32,
    hotkey_choice: i32,
    hd_path: PathBuf,
    out_path: PathBuf,
    voobly_dir: PathBuf,
    up_dir: PathBuf,
    mod_name: String,
}

impl ConvertOptions {
    fn into_ffi(self) -> FFIWKSettings {
        FFIWKSettings {
            use_voobly: self.use_voobly,
            use_exe: self.use_exe,
            use_both: self.use_both,
            use_monks: self.use_monks,
            use_pw: self.use_pw,
            use_walls: self.use_walls,
            copy_maps: self.copy_maps,
            copy_custom_maps: self.copy_custom_maps,
            restricted_civ_mods: self.restricted_civ_mods,
            use_no_snow: self.use_no_snow,
            fix_flags: self.fix_flags,
            replace_tooltips: self.replace_tooltips,
            use_grid: self.use_grid,
            install_directory: encode_path(self.install_directory),
            language: CString::new(self.language).unwrap().as_ptr(),
            dlc_level: self.dlc_level,
            patch: self.patch,
            hotkey_choice: self.hotkey_choice,
            hd_path: encode_path(self.hd_path),
            out_path: encode_path(self.out_path),
            voobly_dir: encode_path(self.voobly_dir),
            up_dir: encode_path(self.up_dir),
            mod_name: CString::new(self.mod_name).unwrap().as_ptr(),
        }
    }
}

pub trait ConvertListener {
    fn finished(&self) {}
    fn log(&self, message: &str) {}
    fn error(&self, message: &str) {
        eprintln!("[wololokingdoms] {}", message);
    }
}

struct Converter {
    ffi_listener: FFIWKListener,
    listener: Arc<Box<ConvertListener>>,
    internal: *mut c_void,
}

impl Converter {
    pub fn new(options: ConvertOptions, listener: Box<ConvertListener>) -> Self {
        let mut settings = options.into_ffi();
        let arc_listener = Arc::new(listener);

        fn get_listener(ptr: *mut c_void) -> Arc<Box<ConvertListener>> {
            Arc::clone(unsafe { mem::transmute(ptr) })
        }

        extern fn finished(data: *mut c_void) {
            let listener = get_listener(data);
            listener.finished();
        }

        extern fn log(data: *mut c_void, message: *const c_char) {
            let listener = get_listener(data);
            let message: &CStr = unsafe { CStr::from_ptr(message) };
            let message: &str = message.to_str().unwrap();
            listener.log(message);
        }

        extern fn set_info(data: *mut c_void, message: *const c_char) {
            let listener = get_listener(data);
        }

        extern fn error(data: *mut c_void, message: *const c_char) {
            let listener = get_listener(data);
            let message: &CStr = unsafe { CStr::from_ptr(message) };
            let message: &str = message.to_str().unwrap();
            listener.error(message);
        }

        extern fn create_dialog(data: *mut c_void, message: *const c_char) {
            let listener = get_listener(data);
        }

        extern fn create_dialog_title(data: *mut c_void, title: *const c_char, message: *const c_char) {
            let listener = get_listener(data);
        }

        extern fn create_dialog_replace(data: *mut c_void, message: *const c_char, a: *const c_char, b: *const c_char) {
            let listener = get_listener(data);
        }

        extern fn set_progress(data: *mut c_void, i: u32) {
            let listener = get_listener(data);
        }

        extern fn install_userpatch(data: *mut c_void, exe: *const c_char, flags: *const *const c_char) {
            let listener = get_listener(data);
        }

        let mut ffi_listener = FFIWKListener {
            data: unsafe { mem::transmute(Arc::clone(&arc_listener)) },
            finished,
            log,
            set_info,
            error,
            create_dialog,
            create_dialog_title,
            create_dialog_replace,
            set_progress,
            install_userpatch,
        };
        let internal = unsafe {
            wkconverter_create(&mut settings, &mut ffi_listener)
        };
        Self {
            listener: arc_listener,
            ffi_listener,
            internal,
        }
    }

    pub fn run(self) {
        unsafe { wkconverter_run(self.internal) };
    }
}
