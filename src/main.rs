use mpris::{LoopStatus, Metadata, PlaybackStatus, Player, PlayerFinder};
use std::env;
use std::fs;
use std::time::Duration;

const PLAYER_FILE: &'static str = "/tmp/current_player";

fn read_player_file() -> Option<String> {
    fs::read_to_string(PLAYER_FILE)
        .ok()
        .map(|v| "org.mpris.MediaPlayer2.".to_owned() + &v)
}

fn write_player_file(player_name: &str) {
    fs::write(
        PLAYER_FILE,
        player_name.split_at("org.mpris.MediaPlayer2.".len()).1,
    )
    .unwrap()
}

/// Read the player file, pick it. Or pick one actice player.
fn pick_player<'a>(players: &mut Vec<Player<'a>>) -> Option<Player<'a>> {
    let player_name = read_player_file().unwrap_or("".to_owned());

    players
        .iter()
        .position(|p| &**p.bus_name() == player_name)
        .or_else(|| {
            players.iter().position(|p| {
                p.get_playback_status()
                    .map(|v| v == PlaybackStatus::Playing)
                    .unwrap_or(false)
            })
        }) // If a player is playing, use it
        .or_else(|| {
            players
                .iter()
                .position(|p| &**p.bus_name() != "org.mpris.MediaPlayer2.mpd")
        }) // Else pick non-mpd if exists
        .or(Some(0)) // Else pick anything
        .and_then(|k| {
            Some({
                let player = players.swap_remove(k);
                if &**player.bus_name() != player_name {
                    write_player_file(player.bus_name());
                }
                player
            })
        })
}

struct Color(&'static str);

fn status_color(state: &PlaybackStatus) -> Color {
    match *state {
        PlaybackStatus::Playing => Color("#00ff00"),
        PlaybackStatus::Paused => Color("#ffa500"),
        PlaybackStatus::Stopped => Color("#ff0000"),
    }
}

fn print_outputs(long: &str, short: &str, color: Color) {
    println!("{}", long);
    println!("{}", short);
    println!("{}", color.0);
}

fn metadata_string(m: &Metadata) -> String {
    format!(
        "{} - {}",
        m.artists()
            .map(|v| v.join("/"))
            .and_then(|v| if v.len() == 0 { None } else { Some(v) })
            .unwrap_or("unknown artist".to_owned()),
        m.title().unwrap_or("untitled")
    )
}

fn duration(position: Option<Duration>, length: Option<Duration>) -> String {
    let fmt = |d: Duration| format!("{:02}:{:02}", d.as_secs() / 60, d.as_secs() % 60);
    format!(
        "{} / {}",
        position.map(fmt).unwrap_or("...".to_owned()),
        length.map(fmt).unwrap_or("...".to_owned()),
    )
}

fn display_player<'a>(player: &Player, other_players: &Vec<Player<'a>>) {
    if other_players.len() == 0 {
        return;
    }
    let (_, player_name) = player.bus_name().split_at("org.mpris.MediaPlayer2.".len());
    let display = format!("<{}>", player_name);
    print_outputs(
        &display,
        &display,
        status_color(&player.get_playback_status().unwrap()),
    );
}

fn action_player(players: &Vec<Player>) -> bool {
    let current_player_pos = read_player_file()
        .and_then(|v| players.iter().position(|p| &**p.bus_name() == &v))
        .unwrap_or(0);

    write_player_file(
        players[match env::var("BLOCK_BUTTON").unwrap_or("".to_owned()).as_ref() {
            "4" => {
                if current_player_pos == 0 {
                    players.len() - 1
                } else {
                    current_player_pos - 1
                }
            }
            "5" => (current_player_pos + 1) % players.len(),
            _ => return false,
        }]
        .bus_name(),
    );
    true
}

fn status(player: &Player) {
    let t = player.get_playback_status().unwrap();
    match t {
        PlaybackStatus::Paused => print_outputs("⏸️ paused", "⏸️", status_color(&t)),
        PlaybackStatus::Stopped => print_outputs("🛑 stopped", "🛑", status_color(&t)),
        PlaybackStatus::Playing => {
            let m = player.get_metadata().unwrap();
            let l = m.length();
            let p = player.get_position().ok();

            print_outputs(
                &format!(
                    "🎹 <span font=\"Sarasa Gothic J 13\">{} [{}]</span>",
                    metadata_string(&player.get_metadata().unwrap()),
                    duration(p, l)
                ),
                "🎹",
                status_color(&t),
            )
        }
    }
}

fn action_status(player: &Player) {
    match env::var("BLOCK_BUTTON").ok() {
        None => return,
        Some(n) => match n.as_ref() {
            "2" => player.stop(),       // Middle
            "3" => player.play_pause(), // Right
            "4" => player.previous(),   // Scroll up
            "5" => player.next(),       // Scroll down
            _ => return,
        },
    }
    .unwrap();
}

fn modes(player: &Player) {
    let t = player.get_playback_status().unwrap();

    if let PlaybackStatus::Playing = t {
        let f = |v: bool, a: &'static str| if v { a } else { "" };
        let l = player.get_loop_status().unwrap();
        let s = player.get_shuffle().unwrap();
        let modes = format!(
            "{}{}{}",
            f(s, "🔀"),
            f(l == LoopStatus::Track, "🔂"),
            f(l == LoopStatus::Playlist, "🔁")
        );
        print_outputs(
            &format!(
                "[{}]",
                if modes.len() > 0 {
                    modes
                } else {
                    "　".to_owned()
                }
            ),
            "",
            status_color(&t),
        )
    }
}

fn action_modes(player: &Player) {
    match env::var("BLOCK_BUTTON").ok() {
        None => return,
        Some(n) => match n.as_ref() {
            "1" => player.checked_set_loop_status(match player.get_loop_status().unwrap() {
                LoopStatus::Playlist => LoopStatus::None,
                _ => LoopStatus::Playlist,
            }), // Left
            "2" => player.checked_set_shuffle(!player.get_shuffle().unwrap()), // Middle
            "3" => player.checked_set_loop_status(match player.get_loop_status().unwrap() {
                LoopStatus::Track => LoopStatus::Playlist,
                _ => LoopStatus::Track,
            }), // Right
            _ => return,
        },
    }
    .unwrap();
}

fn main() {
    let mut players = PlayerFinder::new()
        .expect("Cannot open a player finder!")
        .find_all()
        .expect("Cannot find players");
    players.sort_by(|x, y| x.bus_name().cmp(y.bus_name()));

    if let Some(true) = env::var("MPRIS_MODE").map(|v| v == "player").ok() {
        if action_player(&players) {
            return;
        }
    }

    let player = pick_player(&mut players);

    match player {
        None => return,
        Some(player) => {
            if let Ok(mode) = env::var("MPRIS_MODE") {
                match mode.as_ref() {
                    "player" => display_player(&player, &players),
                    "status" => {
                        status(&player);
                        action_status(&player)
                    }
                    "modes" => {
                        modes(&player);
                        action_modes(&player)
                    }
                    _ => return,
                }
            }
        }
    }
}
