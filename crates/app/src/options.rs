use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use toml_edit::{DocumentMut, value};

//for any setters or big getters to config
//also for the config structs nd all the Serde stuff
pub static MUSIC_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let Some(dir) = dirs::audio_dir() else {
        panic!("Failed to find audio directory");
    };
    dir
});
pub fn init_conf() -> std::io::Result<()> {
    if !Path::new("config.toml").exists() {
        let core = format!("[music]\ndirs = [\"{}\"]\n", MUSIC_DIR.to_string_lossy());
        fs::write("config.toml", core)?;
        //plan: for features just have a function or variable or smth that satisfies a trait bound
        //and is basically the default configuration for that one
    }
    Ok(())
}
