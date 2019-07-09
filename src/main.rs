#![feature(proc_macro_hygiene)]

use std::thread;
use std::path::Path;
use clap::{App, Arg, value_t};
use signal_hook::SIGUSR1;
use signal_hook::iterator::Signals;

mod database;
mod repository;
mod server;

fn main() {
	let matches = App::new("scrap").version("1.0")
		.arg(Arg::with_name("port")
			.long("port")
			.help("Server port")
			.takes_value(true)
			.required(true))
		.arg(Arg::with_name("repo")
			.long("repo")
			.help("Repository directory")
			.takes_value(true)
			.required(true))
		.arg(Arg::with_name("static")
			.long("static")
			.help("Static directory")
			.takes_value(true)
			.required(true))
		.arg(Arg::with_name("uri")
			.long("uri")
			.help("PostgreSQL database URI")
			.takes_value(true)
			.required(true))
		.get_matches();
	let port = value_t!(matches.value_of("port"), u16).unwrap();
	let uri = matches.value_of("uri").unwrap().to_owned();
	let repo_str = matches.value_of("repo").unwrap().to_owned();
	let static_str = matches.value_of("static").unwrap().to_owned();

	let pool = database::connect(&uri)
		.expect("Failed to connect to database");
	repository::load(&Path::new(&repo_str), &Path::new(&static_str), &pool)
		.expect("Failed to load repository");

	let signals = Signals::new(&[SIGUSR1]).unwrap();
	thread::spawn(move || {
		for _signal in signals.forever() {
			let pool = match database::connect(&uri) {
				Ok(pool) => pool,
				Err(e) => {
					eprintln!("Failed to connect to database on signal: {}", e);
					continue;
				},
			};
			match repository::load(&Path::new(&repo_str), &Path::new(&static_str), &pool) {
				Ok(_) => (),
				Err(e) => {
					eprintln!("Failed to load repository on signal: {}", e);
					continue;
				},
			};
		}
	});

	server::run(port, pool);
}
