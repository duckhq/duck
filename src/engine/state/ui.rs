use std::sync::Mutex;

use crate::utils::UI_TITLE;

pub struct UiRepository {
    title: Mutex<String>,
}

impl UiRepository {
    pub fn new() -> Self {
        Self {
            title: Mutex::new(UI_TITLE.to_owned()),
        }
    }

    pub fn title(&self) -> String {
        let guard = self.title.lock().unwrap();
        guard.clone()
    }

    pub fn set_title(&self, title: &str) {
        let mut guard = self.title.lock().unwrap();
        *guard = title.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_default_title_if_not_explicitly_set() {
        // Given
        let repository = UiRepository::new();

        // When
        let title = repository.title();

        // Then
        assert_eq!(UI_TITLE, title);
    }

    #[test]
    fn should_return_correct_title_after_update() {
        // Given
        let repository = UiRepository::new();
        repository.set_title("Foo");

        // When
        let title = repository.title();

        // Then
        assert_eq!("Foo", title);
    }
}
