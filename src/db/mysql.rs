
use log::{debug, error};
use mysql::{Pool, PooledConn, Error};


use crate::configuration::get_db_url;

pub struct MySQL {
    pool: Pool,
}

impl MySQL {

    pub fn new() -> Result<MySQL, Error> {
        let binding = get_db_url();
        let db_url = binding.as_str();
        // let pool = Pool::new(db_url).expect("Could not connect to MySQL db!");
        let pool = Pool::new(db_url);

        match pool {
            Ok(pool) => {
                debug!("MySQL at {db_url}");
                
                Ok(MySQL {
                    pool,
                })
            },
            Err(e) => {
                error!("Could not connect to MySQL db!");
                Err(e)
            },
        }
    }

    pub fn get_connection(&mut self) -> PooledConn {
        self.pool.get_conn().expect("Could not get connection from pool!")
    }

}