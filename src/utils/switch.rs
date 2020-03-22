use std::cell::RefCell;

pub struct Switch {
    value: RefCell<bool>,
}

impl Switch {
    pub fn new(value: bool) -> Self {
        Self {
            value: RefCell::new(value),
        }
    }

    pub fn is_on(&self) -> bool {
        self.is(true)
    }

    pub fn is_off(&self) -> bool {
        self.is(false)
    }

    pub fn turn_on(&self) {
        self.set(true);
    }

    pub fn turn_off(&self) {
        self.set(false);
    }

    fn is(&self, value: bool) -> bool {
        *self.value.borrow() == value
    }

    fn set(&self, value: bool) {
        *self.value.borrow_mut() = value;
    }
}
