use std::ffi::{CStr, CString};
use std::{mem, ptr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
#[cfg(not(target_os = "windows"))]
use std::os::unix::ffi::OsStrExt;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
use libc::{c_char, c_void};

#[repr(C)]
struct FFIWKListener {
    data: *const c_void,
    finished: extern fn(*const c_void),
    log: extern fn(*const c_void, *const c_char),
    set_info: extern fn(*const c_void, *const c_char),
    error: extern fn(*const c_void, *const c_char),
    create_dialog: extern fn(*const c_void, *const c_char),
    create_dialog_title: extern fn(*const c_void, *const c_char, *const c_char),
    create_dialog_replace: extern fn(*const c_void, *const c_char, *const c_char, *const c_char),
    set_progress: extern fn(*const c_void, u32),
    install_userpatch: extern fn(*const c_void, *const c_char, *const *const c_char),
}

extern "C" {
    fn wksettings_create() -> *mut c_void;
    fn wksettings_use_voobly(settings: *mut c_void, val: bool);
    fn wksettings_use_exe(settings: *mut c_void, val: bool);
    fn wksettings_use_both(settings: *mut c_void, val: bool);
    fn wksettings_use_monks(settings: *mut c_void, val: bool);
    fn wksettings_use_short_walls(settings: *mut c_void, val: bool);
    fn wksettings_copy_maps(settings: *mut c_void, val: bool);
    fn wksettings_copy_custom_maps(settings: *mut c_void, val: bool);
    fn wksettings_restricted_civ_mods(settings: *mut c_void, val: bool);
    fn wksettings_use_no_snow(settings: *mut c_void, val: bool);
    fn wksettings_use_grid(settings: *mut c_void, val: bool);
    fn wksettings_fix_flags(settings: *mut c_void, val: bool);
    fn wksettings_hd_path(settings: *mut c_void, val: *const c_char);
    fn wksettings_out_path(settings: *mut c_void, val: *const c_char);
    fn wksettings_voobly_path(settings: *mut c_void, val: *const c_char);
    fn wksettings_up_path(settings: *mut c_void, val: *const c_char);
    fn wksettings_destroy(settings: *mut c_void);

    fn wkconverter_create(settings: *mut c_void, listener: *mut FFIWKListener) -> *mut c_void;
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
fn encode_path(buf: &Path) -> *const c_char {
    buf.as_os_str()
        .as_bytes()
        .as_ptr() as *const c_char
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

    pub fn into_ffi(self) -> *mut c_void {
        let hd_directory = self.hd_directory.expect("Must provide the HD Edition directory");
        let hd_directory = encode_path(&hd_directory);
        let aoc_directory = self.aoc_directory.expect("Must provide the AoC output directory");
        let aoc_directory = encode_path(&aoc_directory);
        let voobly_directory = if self.install_type == InstallType::Voobly || self.install_type == InstallType::Both {
            self.voobly_directory.expect("Must provide a Voobly mod folder for this installation type")
        } else {
            PathBuf::from("/")
        };
        let voobly_directory = encode_path(&voobly_directory);
        let userpatch_directory = if self.install_type == InstallType::UserPatch || self.install_type == InstallType::Both {
            self.userpatch_directory.expect("Must provide a UserPatch mod folder for this installation type")
        } else {
            PathBuf::from("/")
        };
        let userpatch_directory = encode_path(&userpatch_directory);

        let language = CString::new(self.language).unwrap();
        let mod_name = CString::new(self.mod_name).unwrap();

        unsafe {
            let settings = wksettings_create();
            wksettings_use_voobly(settings, self.install_type == InstallType::Voobly);
            wksettings_use_exe(settings, self.install_type == InstallType::UserPatch);
            wksettings_use_both(settings, self.install_type == InstallType::Both);
            wksettings_use_monks(settings, self.use_regional_monks);
            wksettings_use_short_walls(settings, self.use_short_walls);
            wksettings_copy_maps(settings, self.copy_maps);
            wksettings_copy_custom_maps(settings, self.copy_custom_maps);
            wksettings_restricted_civ_mods(settings, self.restricted_civ_mods);
            wksettings_use_no_snow(settings, self.use_no_snow);
            wksettings_use_grid(settings, self.use_grid);
            wksettings_fix_flags(settings, self.fix_flags);
            wksettings_hd_path(settings, hd_directory);
            wksettings_out_path(settings, aoc_directory);
            wksettings_voobly_path(settings, voobly_directory);
            wksettings_up_path(settings, userpatch_directory);
            settings
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

pub struct Converter {
    // ffi_listener: Box<FFIWKListener>,
    listener: Arc<Box<ConvertListener>>,
    internal: *mut c_void,
}

impl Converter {
    pub fn new(options: ConvertOptions, listener: Box<ConvertListener>) -> Self {
        let ffi_settings = options.into_ffi();

        let arc_listener = Arc::new(listener);

        fn get_listener(data: *const c_void) -> Arc<Box<ConvertListener>> {
            let arc_listener_ptr: *const Arc<Box<ConvertListener>> = unsafe { mem::transmute(data) };
            println!("data: {:x}", data as usize);
            Arc::clone(unsafe {
                arc_listener_ptr.as_ref()
            }.unwrap())
        }

        extern fn finished(data: *const c_void) {
            let listener = get_listener(data);
            listener.finished();
        }

        extern fn log(data: *const c_void, message: *const c_char) {
            let listener = get_listener(data);
            let message: &CStr = unsafe { CStr::from_ptr(message) };
            let message: &str = message.to_str().unwrap();
            listener.log(message);
        }

        extern fn set_info(data: *const c_void, message: *const c_char) {
            let listener = get_listener(data);
        }

        extern fn error(data: *const c_void, message: *const c_char) {
            let listener = get_listener(data);
            println!("got listener");
            let message: &CStr = unsafe { CStr::from_ptr(message) };
            println!("CStr::from_ptr {:?}", message.to_str());
            let message = message.to_str().unwrap();
            println!("to_str {:?}", message);
            listener.error(message);
        }

        extern fn create_dialog(data: *const c_void, message: *const c_char) {
            let listener = get_listener(data);
        }

        extern fn create_dialog_title(data: *const c_void, title: *const c_char, message: *const c_char) {
            let listener = get_listener(data);
        }

        extern fn create_dialog_replace(data: *const c_void, message: *const c_char, a: *const c_char, b: *const c_char) {
            let listener = get_listener(data);
        }

        extern fn set_progress(data: *const c_void, i: u32) {
            let listener = get_listener(data);
        }

        extern fn install_userpatch(data: *const c_void, exe: *const c_char, flags: *const *const c_char) {
            let listener = get_listener(data);
        }

        let arc_listener_ptr: *const Arc<Box<ConvertListener>> = &arc_listener;
        let ffi_listener = Box::new(FFIWKListener {
            data: unsafe { mem::transmute(arc_listener_ptr) },
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
            wkconverter_create(ffi_settings, Box::into_raw(ffi_listener))
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
