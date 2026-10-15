use anyhow::Result;
use ck3_tiger::GAME_CONSTS;
use tiger_bin_shared::tiger;

fn main() -> Result<()> {
    let result=tiger(GAME_CONSTS, concat!(env!("CARGO_PKG_VERSION"), ".maintenance.r3"), env!("CARGO_PKG_NAME"));
    if std::env::args().any(|arg| arg=="--fail-on" || arg.starts_with("--fail-on=")) {
        if let Err(error)=&result { eprintln!("{error:#}"); std::process::exit(3); }
    }
    result
}
