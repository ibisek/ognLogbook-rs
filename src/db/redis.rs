
use simple_redis::client::Client;

use crate::configuration::get_redis_url;

pub fn get_client() -> Client {
    simple_redis::create(&get_redis_url()).unwrap()
}
