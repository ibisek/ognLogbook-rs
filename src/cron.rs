use crate::{airfield_manager::AirfieldManager, configuration::AIRFIELDS_FILEPATH};

use self::periodic_timer::PeriodicTimer;

// pub(crate) mod periodic_timer;
mod periodic_timer;

mod flown_distance_calculator;
use flown_distance_calculator::{FlownDistanceCalculator, FDC_RUN_INTERVAL};
mod real_takeoff_lookup;
use real_takeoff_lookup::{RealTakeoffLookup, RTL_RUN_INTERVAL};
mod redis_reaper;
use redis_reaper::{RedisReaper, RR_RUN_INTERVAL};

pub struct CronJobs {
    jobs: Vec<PeriodicTimer>,
}

pub trait CronJob {
    fn cron(&self) -> ();
}

impl CronJobs {
    pub fn new() -> CronJobs {
        CronJobs {
            jobs: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        // let handler: fn() = pokus1;
        // let mut t = PeriodicTimer::new("nazev1".into(), 5, pokus1);
        // let mut t = PeriodicTimer::new("nazev1".into(), 5, || println!("P0KUS2"));
        // t.start();
        // self.jobs.push(t);

        // tl = TowLookup()
        // self.towLookupTimer = PeriodicTimer(TowLookup.RUN_INTERVAL, tl.gliderTowLookup)
        // self.towLookupTimer.start()

        let mut redis_reaper_job = PeriodicTimer::new(
            "Redis Reaper".into(), 
            RR_RUN_INTERVAL, 
            RedisReaper::do_work);
            redis_reaper_job.start();
        self.jobs.push(redis_reaper_job);

        let mut dist_calc_job = PeriodicTimer::new(
            "Flown Distance Calculator".into(), 
            FDC_RUN_INTERVAL, 
            FlownDistanceCalculator::calc_distances);
        dist_calc_job.start();
        self.jobs.push(dist_calc_job);

        let mut dist_calc_job = PeriodicTimer::new(
            "Real Take-off Lookup".into(), 
            RTL_RUN_INTERVAL, 
            RealTakeoffLookup::check_takeoffs);
        dist_calc_job.start();
        self.jobs.push(dist_calc_job);
        
        // eventWatcher = EventWatcher()
        // self.eventWatcherTimer = PeriodicTimer(EventWatcher.RUN_INTERVAL, eventWatcher.processEvents)
        // self.eventWatcherTimer.start()
    }

    pub fn stop(&mut self) {
        for job in self.jobs.iter_mut() {
            job.stop();
        }
    }
}
