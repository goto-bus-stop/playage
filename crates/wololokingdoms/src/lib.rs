use std::ffi::{CStr, CString};
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::Arc;
#[cfg(not(target_os = "windows"))]
use std::os::unix::ffi::OsStrExt;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
use libc::{c_char, c_void};

#[repr(C)]
struct FFIWKSettings {
    use_voobly: bool,
    use_exe: bool,
    use_both: bool,
    use_regional_monks: bool,
    use_small_trees: bool,
    use_short_walls: bool,
    copy_maps: bool,
    copy_custom_maps: bool,
    restricted_civ_mods: bool,
    use_no_snow: bool,
    fix_flags: bool,
    replace_tooltips: bool,
    use_grid: bool,
    language: *const c_char,
    dlc_level: i32,
    patch: i32,
    hotkey_choice: i32,
    hd_directory: *const c_char,
    aoc_directory: *const c_char,
    voobly_directory: *const c_char,
    userpatch_directory: *const c_char,
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
fn encode_path(buf: PathBuf) -> &[u8] {
    unimplemented!()
    // buf.as_os_str()
    //     .encode_wide()
    //     .collect::<&[u16]>()
    //     .as_ptr()
}
#[cfg(not(target_os = "windows"))]
fn encode_path(buf: &Path) -> &[u8] {
    buf.as_os_str()
        .as_bytes()
}

/// Identifies the DLCs that a player has installed.
#[derive(PartialEq, Eq)]
pub enum DlcLevel {
    /// Age of Empires II: The Forgotten
    TheForgotten = 1,
    /// Age of Empires II: African Kingdoms
    AfricanKingdoms = 2,
    /// Age of Empires II: Rise of the Rajas
    RiseOfTheRajas = 3,
}

/// The type of installation to execute.
#[derive(PartialEq, Eq)]
pub enum InstallType {
    /// Install WololoKingdoms as a UserPatch mod.
    UserPatch,
    /// Install WololoKingdoms as a Voobly mod.
    Voobly,
    /// Install WololoKingdoms as both a UserPatch and a Voobly mod, sharing most of the resources.
    Both,
}

pub struct ConvertOptions {
    pub install_type: InstallType,
    pub copy_maps: bool,
    pub copy_custom_maps: bool,
    pub restricted_civ_mods: bool,
    pub fix_flags: bool,
    pub replace_tooltips: bool,
    pub language: String,
    pub dlc_level: DlcLevel,
    pub patch: i32,
    pub hotkey_choice: i32,
    pub hd_directory: Option<PathBuf>,
    pub aoc_directory: Option<PathBuf>,
    pub voobly_directory: Option<PathBuf>,
    pub userpatch_directory: Option<PathBuf>,
    pub mod_name: String,
    // Additional mods
    pub use_regional_monks: bool,
    pub use_small_trees: bool,
    pub use_short_walls: bool,
    pub use_no_snow: bool,
    pub use_grid: bool,
}

impl Default for ConvertOptions {
    fn default() -> ConvertOptions {
        ConvertOptions {
            install_type: InstallType::UserPatch,
            copy_maps: false,
            copy_custom_maps: false,
            use_regional_monks: false,
            use_small_trees: false,
            use_short_walls: false,
            use_grid: false,
            restricted_civ_mods: false,
            use_no_snow: false,
            fix_flags: false,
            replace_tooltips: false,
            language: String::from("en"),
            dlc_level: DlcLevel::TheForgotten,
            patch: 0,
            hotkey_choice: 0,
            hd_directory: None,
            aoc_directory: None,
            voobly_directory: None,
            userpatch_directory: None,
            mod_name: String::from("WololoKingdoms"),
        }
    }
}

impl ConvertOptions {
    pub fn with_aoc_directory(mut self, aoc_directory: &Path) -> Self {
        self.aoc_directory = Some(aoc_directory.to_owned());
        self
    }

    pub fn with_hd_directory(mut self, hd_directory: &Path) -> Self {
        self.hd_directory = Some(hd_directory.to_owned());
        self
    }

    pub fn with_userpatch_directory(mut self, userpatch_directory: &Path) -> Self {
        self.userpatch_directory = Some(userpatch_directory.to_owned());
        self
    }

    pub fn with_voobly_directory(mut self, voobly_directory: &Path) -> Self {
        self.voobly_directory = Some(voobly_directory.to_owned());
        self
    }
}

pub trait ConvertListener {
    fn finished(&self) {}
    fn log(&self, message: &str) {}
    fn error(&self, message: &str) {
        eprintln!("[wololokingdoms] {}", message);
    }
}

pub struct Converter {
    // ffi_listener: Box<FFIWKListener>,
    listener: Arc<Box<ConvertListener>>,
    internal: *mut c_void,
}

impl Converter {
    pub fn new(options: ConvertOptions, listener: Box<ConvertListener>) -> Self {
        let hd_directory = options.hd_directory.expect("Must provide the HD Edition directory");
        let hd_directory = encode_path(&hd_directory);
        let aoc_directory = options.aoc_directory.expect("Must provide the AoC output directory");
        let aoc_directory = encode_path(&aoc_directory);
        let voobly_directory = if options.install_type == InstallType::Voobly || options.install_type == InstallType::Both {
            options.voobly_directory.expect("Must provide a Voobly mod folder for this installation type")
        } else {
            PathBuf::from("/")
        };
        let voobly_directory = encode_path(&voobly_directory);
        let userpatch_directory = if options.install_type == InstallType::UserPatch || options.install_type == InstallType::Both {
            options.userpatch_directory.expect("Must provide a UserPatch mod folder for this installation type")
        } else {
            PathBuf::from("/")
        };
        let userpatch_directory = encode_path(&userpatch_directory);

        let language = CString::new(options.language).unwrap();
        let mod_name = CString::new(options.mod_name).unwrap();

        let ffi_settings = Box::new(FFIWKSettings {
            use_voobly: options.install_type == InstallType::Voobly,
            use_exe: options.install_type == InstallType::UserPatch,
            use_both: options.install_type == InstallType::Both,
            use_regional_monks: options.use_regional_monks,
            use_small_trees: options.use_small_trees,
            use_short_walls: options.use_short_walls,
            copy_maps: options.copy_maps,
            copy_custom_maps: options.copy_custom_maps,
            restricted_civ_mods: options.restricted_civ_mods,
            use_no_snow: options.use_no_snow,
            fix_flags: options.fix_flags,
            replace_tooltips: options.replace_tooltips,
            use_grid: options.use_grid,
            language: language.as_ptr(),
            dlc_level: options.dlc_level as i32,
            patch: options.patch,
            hotkey_choice: options.hotkey_choice,
            hd_directory: hd_directory.as_ptr() as *const c_char,
            aoc_directory: aoc_directory.as_ptr() as *const c_char,
            voobly_directory: voobly_directory.as_ptr() as *const c_char,
            userpatch_directory: userpatch_directory.as_ptr() as *const c_char,
            mod_name: mod_name.as_ptr(),
        });

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

        let ffi_listener = Box::new(FFIWKListener {
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
        });
        eprintln!("create wkconverter");
        let internal = unsafe {
            wkconverter_create(Box::into_raw(ffi_settings), Box::into_raw(ffi_listener))
        };
        eprintln!("created wkconverter");
        Self {
            listener: arc_listener,
            internal,
        }
    }

    pub fn run(self) {
        unsafe { wkconverter_run(self.internal) };
    }
}

impl Drop for Converter {
    fn drop(&mut self) {
    }
}
