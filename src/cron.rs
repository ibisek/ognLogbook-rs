use self::periodic_timer::PeriodicTimer;

// pub(crate) mod periodic_timer;
mod periodic_timer;

mod flown_distance_calculator;
use flown_distance_calculator::{FlownDistanceCalculator};

pub struct CronJobs {
    jobs: Vec<PeriodicTimer>,
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

        // self.rr = RedisReaper()
        // self.redisReaperTimer = PeriodicTimer(RedisReaper.RUN_INTERVAL, self.rr.doWork)
        // self.redisReaperTimer.start()

        // let fdc = FlownDistanceCalculator::new();
        // let mut dist_calc_job = PeriodicTimer::new("Flown Distance Calculator".into(), 10, fdc);
        let mut dist_calc_job = PeriodicTimer::new("Flown Distance Calculator".into(), 10, FlownDistanceCalculator::calc_distances);
        dist_calc_job.start();
        self.jobs.push(dist_calc_job);

        // realTakeoffLookup = RealTakeoffLookup()
        // self.realTakeoffLookupTimer = PeriodicTimer(RealTakeoffLookup.RUN_INTERVAL, realTakeoffLookup.checkTakeoffs)
        // self.realTakeoffLookupTimer.start()

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
