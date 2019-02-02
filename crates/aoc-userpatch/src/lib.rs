use std::process::Command;

const NUM_INSTALL_OPTIONS: usize = 20;

pub struct InstallOptions {
    core: bool,
    savegame_format: bool,
    widescreen: bool,
    windowed_mode: bool,
    upnp: bool,
    multiple_queue: bool,
    original_patrol_delay: bool,
    extend_population_caps: bool,
    replace_snow_with_grass: bool,
    alternate_red: bool,
    alternate_purple: bool,
    alternate_gray: bool,
    left_aligned_interface: bool,
    delink_volume: bool,
    precision_scrolling: bool,
    low_fps: bool,
    extended_hotkeys: bool,
    force_gameplay_features: bool,
    display_ore_resource: bool,
    multiplayer_anti_cheat: bool,
}

impl ToString for InstallOptions {
    fn to_string(&self) -> String {
        let mut string = String::with_capacity(NUM_INSTALL_OPTIONS);

        fn x(val: bool) -> char {
            if val { '1' } else { '0' }
        }

        string.push(x(self.core));
        string.push(x(self.savegame_format));
        string.push(x(self.widescreen));
        string.push(x(self.windowed_mode));
        string.push(x(self.upnp));

        string.push(x(self.multiple_queue));
        string.push(x(self.original_patrol_delay));
        string.push(x(self.extend_population_caps));
        string.push(x(self.replace_snow_with_grass));
        string.push(x(self.alternate_red));
        string.push(x(self.alternate_purple));
        string.push(x(self.alternate_gray));

        string.push(x(self.left_aligned_interface));
        string.push(x(self.delink_volume));
        string.push(x(self.precision_scrolling));
        string.push(x(self.low_fps));
        string.push(x(!self.extended_hotkeys));
        string.push(x(self.force_gameplay_features));
        string.push(x(self.display_ore_resource));
        string.push(x(!self.multiplayer_anti_cheat));

        string
    }
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            core: true,
            savegame_format: true,
            widescreen: true,
            windowed_mode: true,
            upnp: true,
            multiple_queue: true,
            original_patrol_delay: false,
            extend_population_caps: true,
            replace_snow_with_grass: false,
            alternate_red: false,
            alternate_purple: false,
            alternate_gray: false,
            left_aligned_interface: false,
            delink_volume: false,
            precision_scrolling: false,
            low_fps: false,
            extended_hotkeys: true,
            force_gameplay_features: false,
            display_ore_resource: false,
            multiplayer_anti_cheat: true,
        }
    }
}

pub fn install() {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
