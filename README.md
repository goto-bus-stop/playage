# playage

Aspiring cross-platform Age of Empires 2 multiplayer client.

## Aim

The current aim for this project is to provide a simple client that is suitable for casual play on all major platforms, and to provide a modular base to build multiplayer matchmaking clients on.

The setup should be easy, with automated configuration of UserPatch and WololoKingdoms.

## Structure

PlayAge is based on [DPRun][dprun], a command-line tool that starts DirectPlay games. PlayAge uses this tool because it allows us to keep the Windows-only stuff to a minimum–all the other modules in PlayAge can be cross-platform. By only using Wine for the game itself we can ensure a better experience for non-Windows platforms.

DPRun allows implementing DirectPlay Service Providers in the host application (PlayAge in this case). So, we can implement the actual networking code in Rust. We can even reuse connections from the pre-game lobby for this.

...

## Potential Crates

This is very much in flux and sometimes handwavey, but I imagine a list of separated concerns could look like this:

| Name | Purpose |
|------|---------|
| dprun | Runs DPRun with some options |
| dprunsp-libp2p | libp2p based service provider for DPRun |
| playage-matchmaking | Data model and interface crate for matchmaking APIs—defines interfaces like `list_rooms()`, `join_room()` |
| playage-matchmaking-libp2p | A matchmaking API based on a libp2p swarm. This is the vaguest part currently. |
| playage-mod-repository | A crate like this could implement discovery and downloading of game mods |
| aoc-userpatch | Rust API wrapper around the UserPatch SetupAoC.exe CLI (maybe ships with an embedded exe) |
| wololokingdoms | Rust API wrapper around the WololoKingdoms installer library |
| playage | API package that combines all of the above and provides a data model for GUIs |
| playage-gui | Probably a GUI app around the playage API package |

## License

[GPL-3.0](./LICENSE.md)

[dprun]: https://github.com/playage/dprun
