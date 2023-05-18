
use std::collections::{HashSet, HashMap};
use std::sync::{Arc, Once};
use log::warn;

use mysql::Row;
use mysql::prelude::Queryable;
use ogn_client::data_structures::AddressType;

use crate::db::mysql::MySQL;

pub struct PermanentStorage {
    address_type: String,
    entries: HashSet<String>,
}

impl PermanentStorage {
    pub fn new(address_type: String) -> PermanentStorage {
        Self {
            address_type,
            entries: HashSet::new(),
        }
    }

    pub fn reload(&mut self) {
        match MySQL::new() {
            Err(e) => {
                warn!("Could not obtain MySQL connection, skipping reload().");
            },
            Ok(mut mysql) => {
                let mut conn = mysql.get_connection();

                let q = format!("SELECT addr FROM permanent_storage WHERE addr_type='{}' AND active=true", self.address_type);
                let new_entries: Vec<String> = conn.query_map(q, 
                    |mut row: Row| {
                        let addr = row.take("addr").unwrap();
                        addr
                    }
                ).unwrap();

                self.entries.clear();
                for e in new_entries {
                    self.entries.insert(e);
                }
            },
        };
    }

    /// Is specified address' data eligible for permanent storage?
    /// :param address
    pub fn eligible4ps(&self, address: &str) -> bool {
        self.entries.contains(address)
    }
}

pub static mut PSF: PermanentStorageFactory = PermanentStorageFactory::new();
pub static ONCE: Once = Once::new();

pub struct PermanentStorageFactory {
    permanent_storages: Option<HashMap<String, Arc<PermanentStorage>>>,
}

impl PermanentStorageFactory {

    const fn new() -> PermanentStorageFactory {
        Self {
            permanent_storages: None,
        }
    }

    pub fn instance() -> &'static mut PermanentStorageFactory {
        ONCE.call_once(|| {
            unsafe {
                PSF.permanent_storages = Some(HashMap::new()); 
            }
        });

        unsafe { &mut PSF }
    }

    pub fn storage_for(&mut self, address_type: &AddressType) -> Arc<PermanentStorage> {
        let addr_type = address_type.as_short_str();
        if !self.permanent_storages.as_ref().unwrap().contains_key(&addr_type) {
            let mut ps = PermanentStorage::new(addr_type.clone());
            ps.reload();
            self.permanent_storages.as_mut().unwrap().insert(addr_type.clone(), Arc::new(ps));
        }

        self.permanent_storages.as_ref().unwrap()[&addr_type].clone()
    }

    pub fn reload_all(&mut self) {
        // TODO ja nevim :|

        // let x = self.permanent_storages.unwrap().as_ref().lock().unwrap().get_mut("X").unwrap();
        
        // let mut binding = self.permanent_storages.as_mut().unwrap();
        // let mut binding = binding.lock().unwrap();
        // let x = binding.borrow_mut().get_mut("X").unwrap();
        // x.reload();

        // for (key, ps) in self.permanent_storages.unwrap().lock().unwrap().iter_mut() { // all other iterators are consuming!
        //     println!("key: {key}"); 
        //     ps.reload();
        // }

        println!("XXX");
    }
}

#[cfg(test)]
mod tests {
    use ogn_client::data_structures::AddressType;


    #[test]
    fn zkouska1() {
        let psf = super::PermanentStorageFactory::instance();
        let _a = psf.storage_for(&AddressType::Ogn);

        let psf2 = super::PermanentStorageFactory::instance();
        let ps = psf2.storage_for(&AddressType::Ogn);

        let res = ps.eligible4ps("C35001");
        assert_eq!(res, true);

        psf2.reload_all();
    }

}
