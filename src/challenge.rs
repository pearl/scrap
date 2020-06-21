use std::collections::BTreeMap;
use std::fs;
use std::io::{self, ErrorKind::InvalidData};
use std::path::{Path, PathBuf};

use pulldown_cmark::{Parser, Event::Start, Tag::Link, LinkType::Inline, CowStr::Borrowed};
use pulldown_cmark::html::push_html;
use serde::Deserialize;
use tiny_keccak::{Shake, Hasher, Xof};

use crate::ClientPool;

#[derive(Deserialize)]
pub struct Challenge {
	slug: String,
	title: String,
	author: String,
	description: String,
	tags: Vec<String>,
	files: Vec<PathBuf>,
	flag: String,
	enabled: bool,
}

impl Challenge {
	pub fn new(config: &Path, out: &Path) -> io::Result<Self> {
		let mut challenge: Challenge = fs::read_to_string(&config)
			.and_then(|string| toml::from_str(&string)
				.map_err(|err| io::Error::new(InvalidData, err)))?;

		let base = config.parent().unwrap();
		let links: BTreeMap<String, String> = challenge.files.iter()
			.map(|file| base.join(file))
			.map(|file| fs::read(&file)
				.map(|buffer| {
					let mut hash = [0; 8];
					let mut shake = Shake::v256();
					shake.update(&buffer);
					shake.squeeze(&mut hash);
					hex::encode(hash)
				})
				.and_then(|hash| {
					let name = file.file_name().unwrap()
						.to_str().unwrap().to_string();
					let mut path = out.join("files").join(&hash);
					fs::create_dir_all(&path)?;
					path.push(&name);
					fs::copy(file, path)?;
					let link = format!("/static/files/{}/{}", hash, name);
					Ok((name, link))
				}))
			.collect::<io::Result<_>>()?;

		let parser = Parser::new(&challenge.description)
			.map(|event| match event {
				Start(Link(Inline, Borrowed(mut href), title)) => {
					if let Some(link) = links.get(href) {
						href = link;
					}
					Start(Link(Inline, Borrowed(href), title))
				},
				_ => event,
			});

		let mut description = String::new();
		push_html(&mut description, parser);
		challenge.description = description;
		Ok(challenge)
	}

	pub fn push(&self, pool: &ClientPool) -> Result<(), postgres::Error> {
		let mut client = pool.get().unwrap();
		client.execute(
		"INSERT INTO scrap.challenge (slug, title, author, description, tags, flag, enabled)
		VALUES ($1, $2, $3, $4, $5, $6, $7)
		ON CONFLICT (slug) DO UPDATE
		SET title=$2, author=$3, description=$4, tags=$5, flag=$6, enabled=$7",
		&[
			&self.slug,
			&self.title,
			&self.author,
			&self.description,
			&self.tags,
			&self.flag,
			&self.enabled,
		])?;
		Ok(())
	}
}
