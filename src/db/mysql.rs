
use log::debug;
use mysql::{Pool, PooledConn};


use crate::configuration::get_db_url;

pub struct MySQL {
    pool: Pool,
}

impl MySQL {

    pub fn new() -> MySQL {
        let binding = get_db_url();
        let db_url = binding.as_str();
        let pool = Pool::new(db_url).expect("Could not connect to MySQL db!");

        debug!("MySQL at {db_url}");

        MySQL {
            pool,
        }
    }

    pub fn get_connection(&mut self) -> PooledConn {
        self.pool.get_conn().expect("Could not get connection from pool!")
    }

}