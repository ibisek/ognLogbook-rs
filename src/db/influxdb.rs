use rinfluxdb::influxql::blocking::Client;
use url::Url;

use crate::configuration::get_influx_url;

pub fn get_client() -> Client {
    Client::new(Url::parse(&get_influx_url()).unwrap(), Some(("", ""))).unwrap()
}
