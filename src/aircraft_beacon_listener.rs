use chrono::Timelike;
use queues::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;
use std::vec;

use chrono::Utc;
use log::info;

use crate::configuration::get_mqtt_config;
use crate::mqtt::{Mqtt, MqttMessage};

use ogn_client::data_structures::{AircraftBeacon, Observer, AddressType};

pub struct AircraftBeaconListener {
    ogn_q: Arc<Mutex<Queue<AircraftBeacon>>>,
    icao_q: Arc<Mutex<Queue<AircraftBeacon>>>,
    flarm_q: Arc<Mutex<Queue<AircraftBeacon>>>,
    safesky_q: Arc<Mutex<Queue<AircraftBeacon>>>,
    time: SystemTime,
    num_ogn: u64,
    num_icao: u64,
    num_flarm: u64,
    num_sky: u64,
    mqtt: Mqtt,
}

impl AircraftBeaconListener {
    pub fn new(ogn_q: Arc<Mutex<Queue<AircraftBeacon>>>, 
        icao_q: Arc<Mutex<Queue<AircraftBeacon>>>, 
        flarm_q: Arc<Mutex<Queue<AircraftBeacon>>>,
        safesky_q: Arc<Mutex<Queue<AircraftBeacon>>>) -> AircraftBeaconListener {

            let (mqtt_id, mqtt_host, mqtt_port, mqtt_username, mqtt_password) = get_mqtt_config();
            let mqtt = Mqtt::new(&mqtt_id, &mqtt_host, mqtt_port, &mqtt_username, &mqtt_password);

        Self {
            ogn_q,
            icao_q,
            flarm_q,
            safesky_q,
            time: SystemTime::now(),
            num_ogn: 0,
            num_icao: 0,
            num_flarm: 0,
            num_sky: 0,
            mqtt,
        }
    }
}

impl Observer<AircraftBeacon> for AircraftBeaconListener {
    fn notify(&mut self, beacon: AircraftBeacon) {

        if beacon.addr_type == AddressType::Ogn {
            self.ogn_q.lock().unwrap().add(beacon).unwrap();
            self.num_ogn += 1;
        } else 
        if beacon.addr_type == AddressType::Icao {
            self.icao_q.lock().unwrap().add(beacon).unwrap();
            self.num_icao += 1;
        } else 
        if beacon.addr_type == AddressType::Flarm {
            self.flarm_q.lock().unwrap().add(beacon).unwrap();
            self.num_flarm += 1;
        } else
        if beacon.addr_type == AddressType::SafeSky {
            self.safesky_q.lock().unwrap().add(beacon).unwrap();
            self.num_sky += 1;
        } 

        // process and report some stats:
        if self.time.elapsed().unwrap().as_secs() >= 60 {
            let q_len_ogn = self.ogn_q.lock().unwrap().size();
            let q_len_icao = self.icao_q.lock().unwrap().size();
            let q_len_flarm = self.flarm_q.lock().unwrap().size();
            let q_len_sky = self.safesky_q.lock().unwrap().size();
            let beacons_rate = self.num_ogn + self.num_icao + self.num_flarm + self.num_sky;
            let beacons_queued = q_len_ogn + q_len_icao + q_len_flarm + q_len_sky;
            info!("Beacon rate: {}/min, {} queued (O {} / I {} / F {} / S {})", 
                beacons_rate, 
                beacons_queued,
                q_len_ogn, q_len_icao, q_len_flarm, q_len_sky
            );

            const LIMIT: u64 = 1000;
            let mut messages:Vec<MqttMessage> = vec![];
            if beacons_rate > LIMIT {
                messages.push(MqttMessage{topic:"ognLogbookRs/rate".into(), payload: format!("{beacons_rate}")});
                messages.push(MqttMessage{topic:"ognLogbookRs/queued".into(), payload: format!("{beacons_queued}")});
            }
            if  self.num_ogn > LIMIT {
                messages.push(MqttMessage{topic:"ognLogbookRs/ogn".into(), payload: format!("{}", self.num_ogn)});
            }
            if  self.num_icao > LIMIT {
                messages.push(MqttMessage{topic:"ognLogbookRs/icao".into(), payload: format!("{}", self.num_icao)});
            }
            if  self.num_flarm > LIMIT {
                messages.push(MqttMessage{topic:"ognLogbookRs/flarm".into(), payload: format!("{}", self.num_flarm)});
            }
            if  self.num_sky > LIMIT {
                messages.push(MqttMessage{topic:"ognLogbookRs/sky".into(), payload: format!("{}", self.num_sky)});
            }

            self.mqtt.send_mqtt_messages(&messages);

            self.num_ogn = 0;
            self.num_icao = 0;
            self.num_flarm = 0;
            self.num_sky = 0;
            self.time = SystemTime::now();
        }
    }
}
