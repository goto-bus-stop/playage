mod installer;
use std::process::Command;
use std::io::Result;
use std::fs::File;
use installer::extract_installer;

const NUM_INSTALL_OPTIONS: usize = 20;

pub struct InstallOptions {
    widescreen_command_bar: bool,
    windowed_mode: bool,
    upnp: bool,

    alternate_red: bool,
    alternate_purple: bool,
    alternate_gray: bool,
    extend_population_caps: bool,
    replace_snow_with_grass: bool,
    water_animation: bool,
    precision_scrolling: bool,
    shift_group_append: bool,
    keydown_hotkeys: bool,

    savegame_format: bool,
    multiple_queue: bool,
    original_patrol_delay: bool,
    water_movement: bool,
    weather_system: bool,
    custom_terrains: bool,
    terrain_underwater: bool,
    numeric_age_display: bool,
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
        let mut flags = String::with_capacity(NUM_INSTALL_OPTIONS);

        fn x(val: bool) -> char {
            if val { '1' } else { '0' }
        }

        flags.push(x(self.widescreen_command_bar));
        flags.push(x(self.windowed_mode));
        flags.push(x(self.upnp));

        flags.push(x(self.alternate_red));
        flags.push(x(self.alternate_purple));
        flags.push(x(self.alternate_gray));
        flags.push(x(self.extend_population_caps));
        flags.push(x(self.replace_snow_with_grass));
        flags.push(x(self.water_animation));
        flags.push(x(self.precision_scrolling));
        flags.push(x(self.shift_group_append));
        flags.push(x(self.keydown_hotkeys));

        flags.push(x(self.savegame_format));
        flags.push(x(self.multiple_queue));
        flags.push(x(self.original_patrol_delay));
        flags.push(x(!self.water_movement));
        flags.push(x(!self.weather_system));
        flags.push(x(!self.custom_terrains));
        flags.push(x(!self.terrain_underwater));
        flags.push(x(self.numeric_age_display));
        flags.push(x(self.touch_screen_control));
        flags.push(x(self.store_spec_addresses));
        flags.push(x(self.normal_mouse));

        flags.push(x(self.delink_volume));
        flags.push(x(self.wine_chatbox));
        flags.push(x(self.low_quality_environment));
        flags.push(x(self.low_fps));
        flags.push(x(!self.extended_hotkeys));
        flags.push(x(self.force_gameplay_features));
        flags.push(x(self.display_ore_resource));
        flags.push(x(!self.multiplayer_anti_cheat));
        flags.push(x(self.default_background_mode));
        flags.push(x(self.sp_at_multiplayer_speed));
        flags.push(x(self.debug_logging));
        flags.push(x(self.statistics_font_style));
        flags.push(x(self.background_audio_playback));
        flags.push(x(!self.civilian_attack_switch));
        flags.push(x(self.handle_small_farm_selections));
        flags.push(x(self.spec_research_events));

        format!(r#""-i" "-f:{}""#, flags)
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
