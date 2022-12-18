use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use log::{warn, error};
use mysql::*;
use mysql::prelude::*;

use queues::*;

pub struct DbThread {
    thread: Option<thread::JoinHandle<()>>,
    do_run: Arc<AtomicBool>,
    pool: Arc<Pool>,
    to_do_statements: Arc<Mutex<Queue<String>>>,
}

impl DbThread {

    pub fn new(db_url: &str) -> DbThread {
        DbThread {
            thread: None,
            do_run: Arc::new(AtomicBool::new(true)),
            pool: Arc::new(Pool::new(db_url).expect("Could not connect to MySQL db!")),
            to_do_statements: Arc::new(Mutex::new(Queue::new())),
        }
    }

    pub fn stop(&mut self) {
        self.do_run.swap(false, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }

    pub fn add_statement(&mut self, sql: String) {
        self.to_do_statements.lock().unwrap().add(sql).unwrap();
    }

    pub fn start(&mut self) {
        if self.thread.is_some() {
            warn!("Refused to start db_thread. The thread is already running!");
            return;
        }

        // vars used by the thread internally:
        let q = Arc::clone(&self.to_do_statements);
        let do_run = Arc::clone(&self.do_run);
        let pool = Arc::clone(&self.pool);

        let thread = thread::spawn(
            move || {
                while do_run.load(Ordering::Relaxed) {
                    let num_queued = q.lock().unwrap().size();
                    if num_queued == 0 {
                        thread::sleep(Duration::from_millis(500));    
                        continue;
                    }
                    
                    let mut conn = pool.get_conn().expect("Could not get connection from pool!");

                    while q.lock().unwrap().size() > 0 {
                        let sql = q.lock().unwrap().remove().unwrap();
                        match conn.query_drop(&sql) {
                            Ok(_) => (),
                            Err(e) => error!("Error when executing query '{sql}': {e}")
                        };
                    }
                }
        });

        self.thread = Some(thread);
    }

}