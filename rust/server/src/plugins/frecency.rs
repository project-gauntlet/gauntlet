
// shamelessly stolen from MIT licensed https://github.com/camdencheek/fre

use std::time::SystemTime;

pub struct FrecencyMetaParams {
    pub reference_time: f64,
    pub half_life: f64,
}

impl Default for FrecencyMetaParams {
    fn default() -> FrecencyMetaParams {
        FrecencyMetaParams {
            reference_time: current_time_secs(),
            half_life: 60.0 * 60.0 * 24.0 * 3.0, // three day half life
        }
    }
}


#[derive(Clone)]
pub struct FrecencyItemStats {
    pub(in super) half_life: f64,
    pub(in super) reference_time: f64, // Time in seconds since the epoch
    pub(in super) last_accessed: f64, // Time in seconds since reference_time that this item was last accessed
    pub(in super) frecency: f64,
    pub(in super) num_accesses: i32,
}

impl FrecencyItemStats {
    /// Create a new item
    pub fn new(ref_time: f64, half_life: f64) -> FrecencyItemStats {
        FrecencyItemStats {
            half_life,
            reference_time: ref_time,
            frecency: 0.0,
            last_accessed: 0.0,
            num_accesses: 0,
        }
    }

    /// Return the number of half lives passed since the reference time
    pub fn half_lives_passed(&self) -> f64 {
        (current_time_secs() - self.reference_time) / self.half_life
    }

    pub fn mark_used(&mut self) {
        self.update_frecency(1.0);
        self.update_num_accesses(1);
        self.update_last_access(current_time_secs());
    }

    /// Change the half life of the item, maintaining the same frecency
    pub fn set_half_life(&mut self, half_life: f64) {
        let secs = current_time_secs();
        self.reset_ref_time(secs);

        let old_frecency = self.get_frecency(secs);
        self.half_life = half_life;
        self.set_frecency(old_frecency);
    }

    /// Calculate the frecency of the item
    pub fn get_frecency(&self, current_time_secs: f64) -> f64 {
        self.frecency / 2.0f64.powf((current_time_secs - self.reference_time) / self.half_life)
    }

    pub fn set_frecency(&mut self, new: f64) {
        self.frecency = new * 2.0f64.powf((current_time_secs() - self.reference_time) / self.half_life);
    }

    /// update the frecency of the item by the given weight
    pub fn update_frecency(&mut self, weight: f64) {
        let original_frecency = self.get_frecency(current_time_secs());
        self.set_frecency(original_frecency + weight);
    }

    /// Update the number of accesses of the item by the given weight
    pub fn update_num_accesses(&mut self, weight: i32) {
        self.num_accesses += weight;
    }

    /// Update the time the item was last accessed
    pub fn update_last_access(&mut self, time: f64) {
        self.last_accessed = time - self.reference_time;
    }

    /// Reset the reference time and recalculate the last_accessed time
    pub fn reset_ref_time(&mut self, new_time: f64) {
        let original_frecency = self.get_frecency(current_time_secs());
        let delta = self.reference_time - new_time;
        self.reference_time = new_time;
        self.last_accessed += delta;
        self.set_frecency(original_frecency);
    }

    /// Timestamp (in nanoseconds since epoch) of the last access
    pub fn last_access(&self) -> f64 {
        self.reference_time + self.last_accessed
    }
}

fn current_time_secs() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("failed to get system time")
        .as_secs_f64()
}
