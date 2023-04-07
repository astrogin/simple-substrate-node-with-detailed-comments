//! Substrate Node Template CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod benchmarking;
mod cli;
mod command;
mod rpc;

// This file contains the code that starts the node. It initializes the logger, reads the
// command-line arguments, sets up the node configuration, and starts the node.
fn main() -> sc_cli::Result<()> {
	command::run()
}
