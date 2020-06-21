#![feature(proc_macro_hygiene)]

mod challenge;
mod ctf;
mod server;

use std::fs;
use std::path::Path;

use clap::{App, Arg, value_t};
use r2d2_postgres::PostgresConnectionManager;
use r2d2_postgres::postgres::NoTls;
use r2d2_postgres::r2d2::{Pool, PooledConnection};

use crate::challenge::Challenge;
use crate::ctf::Ctf;

type ClientPool = Pool<PostgresConnectionManager<NoTls>>;
type Client = PooledConnection<PostgresConnectionManager<NoTls>>;

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
	let uri = matches.value_of("uri").unwrap();
	let repo_path = Path::new(matches.value_of("repo").unwrap());
	let static_path = Path::new(matches.value_of("static").unwrap());

	let manager = PostgresConnectionManager::new(uri.parse().unwrap(), NoTls);
	let pool = Pool::new(manager).unwrap();
	let schema = include_str!("../scrap.sql");
	pool.get().unwrap().simple_query(schema).unwrap();

	let ctf = Ctf::new(&repo_path.join("ctf.toml")).unwrap();
	ctf.push(&pool).unwrap();

	pool.get().unwrap().simple_query("UPDATE scrap.challenge SET enabled=NULL").unwrap();
	for challenge in fs::read_dir(repo_path).unwrap()
		.filter_map(|entry| entry.ok())
		.map(|entry| entry.path().join("challenge.toml"))
		.filter(|path| path.is_file())
		.map(|path| Challenge::new(&path, &static_path).unwrap()) {
			challenge.push(&pool).unwrap();
		}
	pool.get().unwrap().simple_query("DELETE FROM scrap.challenge WHERE enabled IS NULL").unwrap();

	server::run(port, pool);
}
