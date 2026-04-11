pub mod agents;

pub fn version() -> &'static str {
    include_str!(concat!(env!("OUT_DIR"), "/version.txt"))
}
pub mod cli;
pub mod config;
pub mod tui;
pub mod workspace;
