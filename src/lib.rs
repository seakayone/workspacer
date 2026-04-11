pub mod agents;
pub mod cli;
pub mod config;
pub mod tui;
pub mod workspace;

pub fn version() -> &'static str {
    include_str!(concat!(env!("OUT_DIR"), "/version.txt"))
}
