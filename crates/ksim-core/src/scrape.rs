use std::collections::VecDeque;

use crate::{observation::ScrapeConfig, performance::ServiceSignals, time::SimTime};

/// Metrics pipeline: snapshots are taken on the scrape grid and consumers read the snapshot
/// `lag_scrapes` scrapes behind the newest one.
#[derive(Debug)]
pub(crate) struct MetricsPipeline {
    config: ScrapeConfig,
    history: VecDeque<Vec<ServiceSignals>>,
    published: Vec<ServiceSignals>,
}

impl MetricsPipeline {
    pub(crate) fn new(config: ScrapeConfig, services: usize) -> Self {
        Self {
            config,
            history: VecDeque::new(),
            published: vec![ServiceSignals::default(); services],
        }
    }

    pub(crate) fn scrape(&mut self, now: SimTime, signals: Vec<ServiceSignals>) {
        if !self.is_scrape_time(now) {
            return;
        }
        let retained = self.lag() + 1;
        self.history.push_back(signals);
        while self.history.len() > retained {
            self.history.pop_front();
        }
        if self.history.len() == retained {
            self.published.clone_from(&self.history[0]);
        }
    }

    pub(crate) fn published(&self) -> &[ServiceSignals] {
        &self.published
    }

    fn lag(&self) -> usize {
        self.config.lag_scrapes as usize
    }

    fn is_scrape_time(&self, now: SimTime) -> bool {
        match self.config.resolution.as_millis() {
            0 => true,
            resolution => now.as_millis().is_multiple_of(resolution),
        }
    }
}
