use std::error::Error;
use r2d2_postgres::PostgresConnectionManager;
use r2d2_postgres::postgres::NoTls;
use r2d2_postgres::r2d2::{Pool, PooledConnection};

pub type ClientPool = Pool<PostgresConnectionManager<NoTls>>;
pub type Client = PooledConnection<PostgresConnectionManager<NoTls>>;

pub fn connect(uri: &str) -> Result<ClientPool, Box<dyn Error>> {
	let manager = PostgresConnectionManager::new(uri.parse()?, NoTls);
	let pool = Pool::new(manager)?;
	let schema = include_str!("../scrap.sql");
	let mut client = pool.get()?;
	client.simple_query(schema)?;
	Ok(pool)
}
