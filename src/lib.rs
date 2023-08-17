pub mod timer {
    use std::time::Duration;
    use std::sync::{Arc, Mutex};
    use std::thread::{self, JoinHandle};
    pub struct Timer {
        dur: Duration,
        func: fn(),
        cancel: Arc<Mutex<bool>>,
    }
    impl Timer {
        pub fn new(dur: Duration,func: fn()) -> Timer {
            Timer {
                dur,
                func,
                cancel: Arc::new(Mutex::new(false)),
            }
        }
        
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
    
        pub fn cancel(self){
            *self.cancel.lock().unwrap() = true;
        }
    }    
}