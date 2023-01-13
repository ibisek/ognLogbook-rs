use rumqttc::{MqttOptions, Client, QoS, Connection};
use std::time::Duration;

pub struct MqttMessage {
    pub topic: String,
    pub payload: String,
}

pub struct Mqtt {
    id: String,
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl Mqtt {
    pub fn new(id: &str, host: &str, port: u16, username: &str, password: &str) -> Mqtt {
        Self {
            id: id.into(),
            host: host.into(),
            port,
            username: username.into(),
            password: password.into(),
        }
    }

    fn send_mqtt_message(&mut self, client: &mut Client, topic: &str, payload: &str) {
        let _ = client.publish(topic, QoS::AtLeastOnce, false, payload);
    }
    
    pub fn send_mqtt_messages(&mut self, messages: &Vec<MqttMessage>) {
        let mut mqttoptions = MqttOptions::new(&self.id, &self.host, self.port);
        mqttoptions.set_keep_alive(Duration::from_secs(10));
        mqttoptions.set_credentials(&self.username, &self.password);

        let (mut client, mut connection) = Client::new(mqttoptions, 10);

        for msg in messages {
            self.send_mqtt_message(&mut client, &msg.topic, &msg.payload);
        }
        
        for (i, _notification) in connection.iter().enumerate() {
            // println!("Notification [{i}] = {:?}", &notification);
            if i > messages.len() { // all sent? probably..
                break;
            }
        }
    }
   
}
