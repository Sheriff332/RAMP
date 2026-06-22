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

fn main() {
    storage::core::init::sql_init().expect("Could not initialize database");
    storage::core::init::init_library_sync().expect("Could not initialize library");

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