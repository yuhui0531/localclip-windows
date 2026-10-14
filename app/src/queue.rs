#[derive(Default)]
pub struct Queue {
    pub ids: Vec<String>,
    pub position: usize,
    pub busy: bool,
    pub last_activity: i64,
    pub generation: u64,
    pub shortcut: Option<String>,
    pub stopped: bool,
}
impl Queue {
    pub fn active(&self) -> bool {
        self.position < self.ids.len()
    }
    pub fn start(&mut self, ids: Vec<String>, now: i64) -> Result<(), &'static str> {
        if ids.len() < 2 {
            return Err("selectionTooSmall");
        }
        self.ids = ids;
        self.position = 0;
        self.busy = false;
        self.last_activity = now;
        self.generation += 1;
        Ok(())
    }
    pub fn begin(&mut self, now: i64, has_target: bool) -> bool {
        if !self.active() || self.busy || !has_target {
            return false;
        }
        self.busy = true;
        self.last_activity = now;
        true
    }
    pub fn next_item(&mut self) -> Option<(usize, String)> {
        let id = self.ids.get(self.position)?.clone();
        self.position += 1;
        Some((self.position, id))
    }
    pub fn finish(&mut self, generation: u64) {
        if generation == self.generation {
            self.busy = false;
        }
    }
    pub fn stop(&mut self) {
        self.ids.clear();
        self.position = 0;
        self.busy = false;
        self.generation += 1;
    }
    pub fn expired(&self, now: i64) -> bool {
        self.active() && now.saturating_sub(self.last_activity) >= 600_000
    }
    pub fn document(&self) -> serde_json::Value {
        serde_json::json!({"active":self.active(),"position":self.position,"total":self.ids.len(),"busy":self.busy})
    }
}
