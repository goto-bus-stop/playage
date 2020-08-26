#![allow(clippy::unreadable_literal)]
use crate::{InstallOptions, InterfaceStyle};
use std::{fmt, str};

#[derive(Clone)]
pub struct Feature {
    pub name: &'static str,
    pub optional: bool,
    pub affects_sync: bool,
    enabled: bool,
    patches: &'static [Injection],
}

impl fmt::Debug for Feature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Feature {{ \"{}\", optional: {:?}, affects_sync: {:?}, enabled: {:?} }}",
            self.name, self.optional, self.affects_sync, self.enabled
        )
    }
}

impl Feature {
    fn assert_optional(&self) {
        assert!(
            self.optional,
            "cannot toggle non-optional feature \"{}\"",
            self.name
        );
    }

    pub fn enable(&mut self, enabled: bool) {
        self.assert_optional();
        self.enabled = enabled;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

/// Describes a patch as an offset and a hexadecimal string.
struct Injection(u32, &'static [u8]);

/// Overwrite bytes in buffer at an offset.
fn apply_patch(buffer: &mut [u8], offset: usize, patch: &[u8]) {
    let end = offset + patch.len();
    (&mut buffer[offset..end]).copy_from_slice(&patch);
}

include!(concat!(env!("OUT_DIR"), "/injections.rs"));

fn configure_features(options: &InstallOptions) -> Vec<Feature> {
    FEATURES
        .iter()
        .cloned()
        .map(|mut f| {
            if !f.optional {
                return f;
            }
            f.enable(match f.name {
                "Widescreen interface style" => {
                    options.interface_style == InterfaceStyle::Widescreen
                }
                "Left-aligned interface style" => {
                    options.interface_style == InterfaceStyle::LeftAligned
                }
                "Windowed mode support" => options.windowed_mode,
                "Port forwarding support" => options.upnp,
                "Darken mini-map red" => options.alternate_red,
                "Darken mini-map purple" => options.alternate_purple,
                "Darken mini-map grey" => options.alternate_gray,
                "Population caps to 1000" => options.extend_population_caps,
                "Snow/ice terrain removal" => options.replace_snow_with_grass,
                "Enable water animation" => options.water_animation,
                "Precision scrolling system" => options.precision_scrolling,
                "Shift group appending" => options.shift_group_append,
                "Keydown object hotkeys" => options.keydown_hotkeys,
                "New save filename format" => options.savegame_format,
                "Multiple building queue" => options.multiple_queue,
                "Original patrol default" => options.original_patrol_delay,
                "Disable water movement" => !options.water_movement,
                "Disable weather system" => !options.weather_system,
                "Disable custom terrains" => !options.custom_terrains,
                "Disable terrain underwater" => !options.terrain_underwater,
                "Numeric age display" => options.numeric_age_display,
                "Touch screen control" => options.touch_screen_control,
                "Store Sx spec addresses" => options.store_spec_addresses,
                "Custom normal mouse" => options.normal_mouse,
                "Delink from system volume" => options.delink_volume,
                "Alternate chat box for wine" => options.wine_chatbox,
                "Lower quality environment" => options.low_quality_environment,
                "Restore 20fps for single player" => options.low_fps,
                "Disable extended hotkeys" => !options.extended_hotkeys,
                "Force new gameplay features" => options.force_gameplay_features,
                "Ore resource amount display" => options.display_ore_resource,
                "Disable multiplayer anti-cheat" => !options.multiplayer_anti_cheat,
                "Default to background mode" => options.default_background_mode,
                "Windowed fullscreen mode" => false,
                "Multiplayer single player speed" => options.sp_at_multiplayer_speed,
                "Rms and Scx debug logging" => options.debug_logging,
                "Change statistics font style" => options.statistics_font_style,
                "Background audio playback" => options.background_audio_playback,
                "Disable civilian attack switch" => options.civilian_attack_switch,
                "Handle small farm selections" => options.handle_small_farm_selections,
                "Show rec/spec research events" => options.spec_research_events,
                "Show rec/spec market events" => options.spec_market_events,
                "Disable rec/spec score stats" => !options.spec_score_stats,
                "Hidden civilization selection" => false,
                "Allow spectators by default" => false,
                _ => unreachable!(f.name),
            });
            f
        })
        .collect()
}

/// Install UserPatch 1.5 into a buffer containing a 1.0c executable.
pub fn install_into(exe_buffer: &[u8], options: &InstallOptions) -> Vec<u8> {
    let features = configure_features(options);

    let mut extended_buffer = exe_buffer.to_vec();
    let three_megs = 3 * 1024 * 1024;
    extended_buffer.extend(&vec![0; three_megs - exe_buffer.len()]);

    for feature in features.iter() {
        if !feature.enabled() {
            continue;
        }

        let Feature { patches, .. } = feature;
        for Injection(addr, patch) in patches.iter() {
            let mut addr = *addr as usize;
            if addr > extended_buffer.len() {
                if addr < 0x7A5000 {
                    addr -= 0x400000;
                } else {
                    addr -= 0x512000;
                }
            }
            apply_patch(&mut extended_buffer, addr, patch);
        }
    }
    extended_buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InstallOptions;
    use std::fs::{read, write};

    #[test]
    fn apply_patch_test() {
        let mut buffer = vec![0u8; 256];
        apply_patch(&mut buffer, 8, &[1u8; 8]);
        assert_eq!(
            &buffer[0..24],
            &[
                0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8,
                0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            ]
        );
        apply_patch(&mut buffer, 10, &[2u8; 4]);
        assert_eq!(
            &buffer[0..24],
            &[
                0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 1u8, 2u8, 2u8, 2u8, 2u8, 1u8, 1u8,
                0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            ]
        );
    }

    #[test]
    fn produce_bare_up15() {
        use std::{env, path::PathBuf};
        if let Ok(base) = env::var("AOCDIR") {
            let base = PathBuf::from(base);
            let aoc = read(base.join("Age2_x1/age2_x1.0c.exe")).unwrap();
            let up15 = install_into(&aoc, &InstallOptions::bare());
            write(base.join("Age2_x1/age2_x1.rs.exe"), &up15).unwrap();
        }
    }
}
