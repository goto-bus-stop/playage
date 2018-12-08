# playage

Cross-platform Age of Empires 2 multiplayer client.

## Structure

PlayAge is based on [DPRun][dprun], a command-line tool that starts DirectPlay games. PlayAge uses this tool because it allows us to keep the Windows-only stuff to a minimum–all the other modules in PlayAge can be cross-platform. By only using Wine for the game itself we can ensure a better experience for non-Windows platforms.

DPRun allows implementing DirectPlay Service Providers in the host application (PlayAge in this case). So, we can implement the actual networking code in Rust. We can even reuse connections from the pre-game lobby for this.

## License

[GPL-3.0](./LICENSE.md)

[dprun]: https://github.com/playage/dprun
