/**
 * !! EXPERIMENTAL !!
 * 
 * This is to pass INFLUXDB insert queries over a Pyton bridge
 * as the RUST influx client is useless due to its inability 
 * to pass multiple semicolon separated requests and needs to 
 * open a separate connection for each query which makes the 
 * insertions hell slow and the overall CPU load too high.
 */

 use log::error;
 use reqwest::header;
// use serde::{Serialize, Deserialize};

const HOST: &str = "localhost";
const PORT: u16 = 8000;

// #[derive(Serialize, Deserialize, Debug)]
// pub struct BridgeResponse {
//     n: u64,
// }

#[derive(Debug)]
pub struct PythonInfluxBridge {}

impl PythonInfluxBridge {
    pub fn new() -> PythonInfluxBridge {
        Self {}
    }

    pub fn insert_into(db_name: &str, queries: String) -> u64 {

        let mut headers = header::HeaderMap::new();
        headers.insert("db_name", header::HeaderValue::from_str(&db_name).unwrap());
    
        let client = reqwest::blocking::Client::builder()
             .default_headers(headers)
             .build().unwrap();
    
        let url = format!("http://{}:{}/insert", HOST, PORT);
        let res = client.post(url).body(queries).send();
        match res {
            Err(e) => error!("Unable to connect to python influx bridge: {}", e),
            Ok(resp) => {
                let data: String = resp.json().unwrap();
                let json: serde_json::Value = serde_json::from_str(&data).expect(&format!("Could not parse json from '{data}'!"));

                let num_accepted = match json.get("n") {
                    Some(val) => match val.as_u64() {
                        Some(val) => val,
                        None => 0,
                    },
                    None => 0,
                };
                
                return num_accepted;
            },
        };

        0
    }

}