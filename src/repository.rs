use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use chrono::DateTime;
use pulldown_cmark::{CowStr, Event, LinkType, Parser, Tag};
use pulldown_cmark::html::push_html;
use serde::Deserialize;
use tiny_keccak::sha3_256;

use crate::database::ClientPool;

#[derive(Deserialize)]
struct Ctf {
	title: String,
	home: String,
	start: Option<toml::value::Datetime>,
	stop: Option<toml::value::Datetime>,
}

#[derive(Deserialize)]
struct Challenge {
	slug: String,
	title: String,
	author: String,
	description: String,
	tags: Vec<String>,
	files: Vec<String>,
	flag: String,
	enabled: bool,
}

pub fn load(repo_path: &Path, static_path: &Path, pool: &ClientPool) -> Result<(), Box<dyn Error>> {
	let mut client = pool.get()?;

	let config = fs::read_to_string(repo_path.join("ctf.toml"))?;
	let ctf: Ctf = toml::from_str(&config)?;
	let parser = Parser::new(&ctf.home);
	let mut home = String::new();
	push_html(&mut home, parser);
	client.simple_query("SELECT setval(pg_get_serial_sequence('scrap.challenge', 'id'), max(id)) FROM scrap.challenge")?;
	client.execute("INSERT INTO scrap.ctf (title, home, start, stop) VALUES ($1, $2, $3, $4)
		ON CONFLICT (id) DO UPDATE SET title=$1, home=$2, start=$3, stop=$4",
		&[
			&ctf.title,
			&home,
			&ctf.start.map(|start| DateTime::parse_from_rfc3339(&start.to_string()).unwrap()),
			&ctf.stop.map(|stop| DateTime::parse_from_rfc3339(&stop.to_string()).unwrap()),
		]
	)?;

	let mut hashes = HashSet::new();
	let static_files_path = static_path.join("files");
	fs::create_dir_all(&static_files_path)?;
	client.simple_query("UPDATE scrap.challenge SET enabled=NULL")?;
	for challenge_path in fs::read_dir(repo_path)?
		.filter_map(|entry| entry.ok())
		.map(|entry| entry.path())
		.filter(|path| path.join("challenge.toml").is_file()) {

		let config = fs::read_to_string(challenge_path.join("challenge.toml"))?;
		let challenge: Challenge = toml::from_str(&config)?;

		let mut url_map = HashMap::new();
		for file_path in challenge.files.iter()
			.map(|path| challenge_path.join(path)) {
			let hash = hex::encode(&sha3_256(&fs::read(&file_path)?)[..8]);
			let hash_path = static_files_path.join(&hash);
			let file_name = String::from(file_path.file_name().unwrap().to_string_lossy());
			let url = format!("/static/files/{}/{}", hash, &file_name);
			fs::create_dir_all(&hash_path)?;
			fs::copy(&file_path, hash_path.join(&file_name))?;
			url_map.insert(file_name, url);
			hashes.insert(OsString::from(hash));
		}

		let parser = Parser::new(&challenge.description)
			.map(|event| match event {
				Event::Start(Tag::Link(LinkType::Inline, href, title)) => {
					match url_map.get(&href.to_string()) {
						Some(url) => Event::Start(Tag::Link(LinkType::Inline, CowStr::Borrowed(url), title)),
						None => Event::Start(Tag::Link(LinkType::Inline, href, title)),
					}
				},
				_ => event,
			});
		let mut description = String::new();
		push_html(&mut description, parser);

		client.execute(
		"INSERT INTO scrap.challenge (slug, title, author, description, tags, flag, enabled)
		VALUES ($1, $2, $3, $4, $5, $6, $7)
		ON CONFLICT (slug) DO UPDATE
		SET title=$2, author=$3, description=$4, tags=$5, flag=$6, enabled=$7",
		&[
			&challenge.slug,
			&challenge.title,
			&challenge.author,
			&description,
			&challenge.tags,
			&challenge.flag,
			&challenge.enabled,
		])?;
	}
	client.simple_query("DELETE FROM scrap.challenge WHERE enabled IS NULL")?;
	for hash_path in fs::read_dir(static_files_path)?
		.filter_map(|entry| entry.ok())
		.filter(|entry| !hashes.contains(&entry.file_name()))
		.map(|entry| entry.path())
		.filter(|path| path.is_dir()) {

		fs::remove_dir_all(hash_path)?;
	}

	Ok(())
}
