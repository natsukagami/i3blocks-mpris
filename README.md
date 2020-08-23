# i3blocks-mpris

Shows MPRIS status on the i3blocks bar.

## Usage

Compile the package with
```sh
cargo build --release
```

There are 3 modes to the executable, chosen by the `MPRIS_MODE` environment variable:
- `player`: Shows the currently selected MPRIS player. If only one is active, it will hide itself away.
  Scrolling on this item will change the active player. 
- `modes`: Shows shuffle/repeat modes. Clicking on this would:
    - Left click: Toggling between no repeat and playlist repeat (repeat all).
    - Middle click: Toggling shuffle.
    - Right click: Toggling between playlist repeat (repeat all) and track repeat (repeat one).
- `status`: Shows the track status (play/pause/stop, track name - track artist, time)
    - Middle click: Stop
    - Right click: Toggle play/pause.
    - Scrolling: Previous / Next.

## Example configuration

```ini
[mpd/player]
command=MPRIS_MODE=player i3blocks-mpris/target/release/mpris
interval=1
signal=10
separator=false
separator_block_width=10

[mpd/modes]
command=MPRIS_MODE=modes i3blocks-mpris/target/release/mpris
interval=1
signal=10
separator=false
separator_block_width=10

[mpd]
command=MPRIS_MODE=status i3blocks-mpris/target/release/mpris
interval=1
signal=10
markup=pango
```

## License 

GNU General Public License, version 2
