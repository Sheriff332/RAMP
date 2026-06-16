// slint::include_modules!();

mod io;
mod logic;
mod storage;

// fn main() -> Result<(), slint::PlatformError> {
//     // MainWindow is generated from app-window.slint
//     let main_window = MainWindow::new()?;
//
//     main_window.run()
// }

use std::io as std_io;
use std::io::Write;
use rusqlite::params;

fn main() {
    sql_init().expect("sqlite initialization failed");

    println!("========================");
    println!("  RAMP (HEADLESS MODE) ");
    println!("========================");
    println!("Type 'help', 'status', or 'exit'.");

    let mut input_line = String::new();

    loop {
        print!("> ");
        std_io::stdout().flush().unwrap();

        std_io::stdin().read_line(&mut input_line).unwrap();
        let input = input_line.trim();

        match input {
            "exit" => break,
            "status" => {
                println!("Status:");
            }
            "help" => {
                println!("Commands:");
            }
            _ => println!("Invalid command"),
        }
        input_line.clear();
    }

}

struct TrackId(u64);
struct ArtistId(u64);
struct CollectionId(u64);
struct TrackSourceId(u64);

struct Queue {
    tracks: Vec<TrackId>,
}

struct PlaybackSession {
    playing: Option<TrackId>,
}

use rusqlite::{Connection, Result};
fn sql_init() -> Result<(), rusqlite::Error> {
    let conn = Connection::open("core.sqlite")?;

    conn.execute("PRAGMA foreign_keys = ON", params![])?;

    conn.execute("CREATE TABLE IF NOT EXISTS tracks (\
    TrackId INTEGER PRIMARY KEY, \
    TrackTitle TEXT NOT NULL, \
    Duration INTEGER NOT NULL, \
    TrackNumber INTEGER, \
    DiscNumber INTEGER DEFAULT 1, \
    Bitrate INTEGER, \
    SampleRate INTEGER, \
    Genre TEXT, \
    Year INTEGER, \
    ImagePath TEXT, \
    Created DATETIME NOT NULL, \
    Updated DATETIME NOT NULL)
    ", params![]
    )?;

    conn.execute("CREATE TABLE IF NOT EXISTS artists (\
    ArtistId INTEGER PRIMARY KEY, \
    ArtistName TEXT NOT NULL, \
    ImagePath TEXT)
    ", params![]
    )?;

    conn.execute("CREATE TABLE IF NOT EXISTS track_artists (\
    TrackId INTEGER NOT NULL, \
    ArtistId INTEGER NOT NULL, \
    PRIMARY KEY (TrackId, ArtistId), \
    FOREIGN KEY (TrackId) REFERENCES tracks(TrackId) ON DELETE CASCADE, \
    FOREIGN KEY (ArtistId) REFERENCES artists(ArtistId) ON DELETE CASCADE)
    ", params![]
    )?;

    conn.execute("CREATE TABLE IF NOT EXISTS collections ( \
    CollectionId INTEGER PRIMARY KEY, \
    CollectionTitle TEXT NOT NULL, \
    CollectionType TEXT NOT NULL, \
    IsUserGenerated INTEGER DEFAULT 0, \
    Created DATETIME NOT NULL, \
    Updated DATETIME NOT NULL, \
    ImagePath TEXT)
    ", params![]
    )?;

    conn.execute("CREATE TABLE IF NOT EXISTS collection_tracks (\
    CollectionId INTEGER NOT NULL, \
    TrackId INTEGER NOT NULL, \
    Position INTEGER NOT NULL, \
    PRIMARY KEY (CollectionId, TrackId), \
    FOREIGN KEY (TrackId) REFERENCES tracks(TrackId) ON DELETE CASCADE, \
    FOREIGN KEY (CollectionId) REFERENCES collections(CollectionId) ON DELETE CASCADE)
    ", params![]
    )?;

    conn.execute("CREATE TABLE IF NOT EXISTS collection_artists (\
    CollectionId INTEGER NOT NULL, \
    ArtistId INTEGER NOT NULL, \
    PRIMARY KEY (CollectionId, ArtistId), \
    FOREIGN KEY (CollectionId) REFERENCES collections(CollectionId) ON DELETE CASCADE, \
    FOREIGN KEY (ArtistId) REFERENCES artists(ArtistId) ON DELETE CASCADE)
    ", params![]
    )?;

    conn.execute("CREATE TABLE IF NOT EXISTS track_sources ( \
    TrackSourceId INTEGER PRIMARY KEY, \
    TrackId INTEGER NOT NULL, \
    Source TEXT NOT NULL, \
    Path TEXT NOT NULL, \
    SourceIdentifier TEXT NOT NULL, \
    FOREIGN KEY (TrackId) REFERENCES tracks(TrackId) ON DELETE CASCADE \
    )", params![]
    )?;

    Ok(())
}