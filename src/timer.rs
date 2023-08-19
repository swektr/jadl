use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
// probably not a good way of making a Timer, but it works
pub struct Timer {
    dur: Duration,
    func: fn(),
    cancel: Arc<Mutex<bool>>,
}
impl Timer {
    /// Create a new Timer that runs some function(func) after some duration(dur).
    pub fn new(dur: Duration,func: fn()) -> Timer {
        Timer {
            dur,
            func,
            cancel: Arc::new(Mutex::new(false)),
        }
    }
    /// Start a Timer, run its func after its dur
    pub fn start(&self) -> JoinHandle<()> {
        let cancel = Arc::clone(&self.cancel);
        let dur = self.dur.clone();
        let func = self.func.clone();
        thread::spawn(move || {
            thread::sleep(dur);
            let cancel = *cancel.lock().unwrap();
            if ! cancel {
                func();
            }
            
        }) 
    }
    /// Tells a started Timer not to run its func then drops the Timer struct;
    pub fn cancel(self){
        *self.cancel.lock().unwrap() = true;
    }
}