mod io;
mod logic;
mod storage;

fn main() {
    let pool = storage::core::innit::get_connection_pool("ramp.sqlite");
    storage::core::innit::init_library(pool.get().expect("Failed to get connection from pool"))
        .expect("Failed to initialize library");
}
