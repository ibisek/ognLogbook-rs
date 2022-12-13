use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::Debug;

use chrono::Utc;

struct EDValue<T> {
    val: T,
    ts: i64,
}

impl <T> EDValue<T> {
    pub fn new(val: T, ts:i64) -> EDValue<T> {
        EDValue {
            val,
            ts,
        }
    }
}

pub struct ExpiringDict <T:Eq+Hash+Clone, U> {
    dict: HashMap<T, EDValue<U>>,
    ttl: i64,
    last_tick_ts: i64,
}

impl <T:Eq+Hash+Clone+Debug, U> ExpiringDict<T, U> {
    pub fn new(ttl: i64) -> ExpiringDict<T, U> {
        ExpiringDict {
            dict: HashMap::new(),
            ttl,
            last_tick_ts: 0,
        }
    }

    pub fn insert(&mut self, key: T, val: U) {
        let ts = Utc::now().timestamp_millis();
        let value = EDValue::new(val, ts);
        self.dict.insert(key, value);
    }

    pub fn contains_key(&self, key: &T) -> bool {
        self.dict.contains_key(key)
    }

    pub fn get(&mut self, key: &T) -> Option<&U> {
        let val = match self.dict.get(key) {
            Some(v) => Some(&v.val),
            None => None,
        };

        val
    }

    // This needs to be called periodically from <somewhere> to drop expired records.
    pub fn tick(&mut self) {
        let now = Utc::now().timestamp_millis();
        if now - self.last_tick_ts < self.ttl { return; }
        self.last_tick_ts = now;

        let keys: Vec<T> = self.dict.keys().cloned().collect();
        for key in keys {
            let ts = self.dict.get(&key).unwrap().ts;
            if now - ts > self.ttl {
                self.dict.remove(&key);
            }
        }
    }
}