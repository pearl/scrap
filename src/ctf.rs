use std::fs;
use std::io::{self, ErrorKind::InvalidData};
use std::path::Path;

use chrono::{DateTime, FixedOffset};
use pulldown_cmark::Parser;
use pulldown_cmark::html::push_html;
use r2d2_postgres::postgres;
use serde::{Deserialize, de};

use crate::ClientPool;

fn rfc3339<'de, D>(deserializer: D) -> Result<Option<DateTime<FixedOffset>>, D::Error>
where
	D: de::Deserializer<'de>,
{
	let datetime: toml::value::Datetime = de::Deserialize::deserialize(deserializer)?;
	Ok(Some(DateTime::parse_from_rfc3339(&datetime.to_string()).unwrap()))
}

#[derive(Debug, Deserialize)]
pub struct Ctf {
	title: String,
	home: String,
	#[serde(default, deserialize_with = "rfc3339")]
	start: Option<DateTime<FixedOffset>>,
	#[serde(default, deserialize_with = "rfc3339")]
	stop: Option<DateTime<FixedOffset>>,
}

impl Ctf {
	pub fn new(config: &Path) -> io::Result<Self> {
		let mut ctf: Ctf = fs::read_to_string(&config)
			.and_then(|string| toml::from_str(&string)
				.map_err(|err| io::Error::new(InvalidData, err)))?;

		let parser = Parser::new(&ctf.home);
		let mut home = String::new();
		push_html(&mut home, parser);
		ctf.home = home;
		Ok(ctf)
	}

	pub fn push(&self, pool: &ClientPool) -> Result<(), postgres::Error> {
		let mut client = pool.get().unwrap();
		client.simple_query("SELECT setval(pg_get_serial_sequence('scrap.challenge', 'id'), max(id)) FROM scrap.challenge")?;
		client.execute("INSERT INTO scrap.ctf (title, home, start, stop) VALUES ($1, $2, $3, $4)
			ON CONFLICT (id) DO UPDATE SET title=$1, home=$2, start=$3, stop=$4",
			&[
				&self.title,
				&self.home,
				&self.start,
				&self.stop,
			]
		)?;
		Ok(())
	}
}
