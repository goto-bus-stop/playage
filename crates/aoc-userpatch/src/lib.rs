mod installer;
mod patch;

use installer::extract_installer;
use std::fs::File;
use std::io::Result;

pub use patch::install_into;

pub struct InstallOptions {
    widescreen_command_bar: bool,
    /// Install windowed mode patch (Windows only).
    windowed_mode: bool,
    /// Install upnp for automatic port forwarding (Windows only).
    upnp: bool,

    /// Use the alternate dark red minimap color.
    alternate_red: bool,
    /// Use the alternate dark purple minimap color.
    alternate_purple: bool,
    /// Use the alternate dark gray minimap color.
    alternate_gray: bool,
    /// Extend max population cap to 1000.
    extend_population_caps: bool,
    /// Replace snow terrains with grass.
    replace_snow_with_grass: bool,
    /// Enable animated water.
    water_animation: bool,
    /// Snap scrolling to pixels, instead of half-tiles.
    precision_scrolling: bool,
    /// Hold shift to append to a numbered unit group.
    shift_group_append: bool,
    /// Trigger hotkeys on keydown instead of keyup.
    keydown_hotkeys: bool,

    /// Use the new savegame file name format.
    savegame_format: bool,
    /// Enable multiple building queueing.
    multiple_queue: bool,
    /// Use the original patrol delay.
    original_patrol_delay: bool,
    water_movement: bool,
    /// Enable the weather system, for rain/snow effects.
    weather_system: bool,
    /// Enable loading custom terrains from scenarios and ZR@ maps.
    custom_terrains: bool,
    terrain_underwater: bool,
    /// Show ages as numbers instead of words in the score display.
    numeric_age_display: bool,
    /// Handle touch screen input events.
    touch_screen_control: bool,
    store_spec_addresses: bool,
    normal_mouse: bool,

    delink_volume: bool,
    wine_chatbox: bool,
    low_quality_environment: bool,
    low_fps: bool,
    extended_hotkeys: bool,
    force_gameplay_features: bool,
    display_ore_resource: bool,
    multiplayer_anti_cheat: bool,
    default_background_mode: bool,
    sp_at_multiplayer_speed: bool,
    debug_logging: bool,
    statistics_font_style: bool,
    background_audio_playback: bool,
    civilian_attack_switch: bool,
    handle_small_farm_selections: bool,
    spec_research_events: bool,
}

impl ToString for InstallOptions {
    fn to_string(&self) -> String {
        let flag_list = [
            self.widescreen_command_bar,
            self.windowed_mode,
            self.upnp,
            self.alternate_red,
            self.alternate_purple,
            self.alternate_gray,
            self.extend_population_caps,
            self.replace_snow_with_grass,
            self.water_animation,
            self.precision_scrolling,
            self.shift_group_append,
            self.keydown_hotkeys,
            self.savegame_format,
            self.multiple_queue,
            self.original_patrol_delay,
            !self.water_movement,
            !self.weather_system,
            !self.custom_terrains,
            !self.terrain_underwater,
            self.numeric_age_display,
            self.touch_screen_control,
            self.store_spec_addresses,
            self.normal_mouse,
            self.delink_volume,
            self.wine_chatbox,
            self.low_quality_environment,
            self.low_fps,
            !self.extended_hotkeys,
            self.force_gameplay_features,
            self.display_ore_resource,
            !self.multiplayer_anti_cheat,
            self.default_background_mode,
            self.sp_at_multiplayer_speed,
            self.debug_logging,
            self.statistics_font_style,
            self.background_audio_playback,
            !self.civilian_attack_switch,
            self.handle_small_farm_selections,
            self.spec_research_events,
        ];

        let flags = flag_list
            .iter()
            .map(|value| if *value { '1' } else { '0' })
            .collect::<String>();

        format!(r#""-i" "-f:{}""#, flags)
    }
}

impl InstallOptions {
    pub fn bare() -> Self {
        Self {
            widescreen_command_bar: false,
            windowed_mode: false,
            upnp: false,
            alternate_red: false,
            alternate_purple: false,
            alternate_gray: false,
            extend_population_caps: false,
            replace_snow_with_grass: false,
            water_animation: false,
            precision_scrolling: false,
            shift_group_append: false,
            keydown_hotkeys: false,
            savegame_format: false,
            multiple_queue: false,
            original_patrol_delay: false,
            water_movement: false,
            weather_system: false,
            custom_terrains: false,
            terrain_underwater: false,
            numeric_age_display: false,
            touch_screen_control: false,
            store_spec_addresses: false,
            normal_mouse: false,
            delink_volume: false,
            wine_chatbox: false,
            low_quality_environment: false,
            low_fps: false,
            extended_hotkeys: false,
            force_gameplay_features: false,
            display_ore_resource: false,
            multiplayer_anti_cheat: false,
            default_background_mode: false,
            sp_at_multiplayer_speed: false,
            debug_logging: false,
            statistics_font_style: false,
            background_audio_playback: false,
            civilian_attack_switch: false,
            handle_small_farm_selections: false,
            spec_research_events: false,
        }
    }
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            widescreen_command_bar: true,
            // Doesn't work in Wine
            windowed_mode: cfg!(os = "windows"),
            upnp: false,

            alternate_red: false,
            alternate_purple: false,
            alternate_gray: false,
            extend_population_caps: true,
            replace_snow_with_grass: false,
            water_animation: true,
            precision_scrolling: true,
            shift_group_append: true,
            keydown_hotkeys: true,

            savegame_format: true,
            multiple_queue: false,
            original_patrol_delay: false,
            water_movement: true,
            weather_system: true,
            custom_terrains: true,
            terrain_underwater: true,
            numeric_age_display: false,
            touch_screen_control: true,
            store_spec_addresses: true,
            normal_mouse: false,

            delink_volume: false,
            // Prevent flicker by default
            wine_chatbox: !cfg!(os = "windows"),
            low_quality_environment: false,
            low_fps: false,
            extended_hotkeys: true,
            force_gameplay_features: false,
            display_ore_resource: false,
            multiplayer_anti_cheat: true,
            default_background_mode: false,
            sp_at_multiplayer_speed: false,
            debug_logging: false,
            statistics_font_style: false,
            background_audio_playback: false,
            civilian_attack_switch: false,
            handle_small_farm_selections: false,
            spec_research_events: false,
        }
    }
}

pub fn install(options: InstallOptions) -> Result<()> {
    {
        let mut file = File::create("/tmp/up.exe")?;
        extract_installer(&mut file)?;
    }

    println!(r#"wine "/tmp/up.exe" {}"#, options.to_string());

    std::fs::remove_file("/tmp/up.exe")?;
    unimplemented!();
}
