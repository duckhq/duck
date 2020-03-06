use std::sync::Mutex;
pub struct State {
    pub title: Mutex<String>,
}

impl State {
    pub fn new() -> Self {
        Self {
            title: Mutex::new("Duck".to_owned()),
        }
    }

    pub fn title<T : Into<String>>(&self, new_title: T) {
        let mut title = self.title.lock().unwrap();
        *title = new_title.into();
    }
}
