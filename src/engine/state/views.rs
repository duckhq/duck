use std::collections::HashSet;
use std::sync::Mutex;

use crate::config::ViewConfiguration;

pub struct ViewRepository {
    views: Mutex<Vec<ViewConfiguration>>,
}

impl ViewRepository {
    pub fn new() -> Self {
        Self {
            views: Mutex::new(Vec::new()),
        }
    }

    pub fn add_views(&self, views: &[ViewConfiguration]) {
        let mut guard = self.views.lock().unwrap();
        guard.clear();
        for view in views.iter() {
            guard.push(view.clone());
        }
    }

    pub fn get_collectors(&self, view_id: &str) -> Option<HashSet<String>> {
        let guard = self.views.lock().unwrap();
        let view = guard.iter().find(|&x| x.id == view_id);
        if let Some(view) = view {
            let mut result = HashSet::<String>::new();
            for collector in view.collectors.iter() {
                result.insert(collector.clone());
            }
            return Some(result);
        }
        None
    }

    pub fn get_views(&self) -> Vec<ViewConfiguration> {
        let guard = self.views.lock().unwrap();
        guard.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Configuration;
    use crate::utils::text::TestVariableProvider;

    #[test]
    fn should_return_collectors_for_view() {
        // Given
        let repository = ViewRepository::new();
        repository.add_views(
            &Configuration::from_json(
                &TestVariableProvider::new(),
                r#"
                { 
                    "collectors": [ 
                        {  "duck": { "id": "a1", "serverUrl": "http://localhost/a1" } },
                        {  "duck": { "id": "a2", "serverUrl": "http://localhost/a2" } },
                        {  "duck": { "id": "b1", "serverUrl": "http://localhost/b1" } },
                        {  "duck": { "id": "b2", "serverUrl": "http://localhost/b2" } },
                        {  "duck": { "id": "b3", "serverUrl": "http://localhost/b3" } },
                        {  "duck": { "id": "c1", "serverUrl": "http://localhost/c1" } },
                        {  "duck": { "id": "c2", "serverUrl": "http://localhost/c2" } },
                        {  "duck": { "id": "c3", "serverUrl": "http://localhost/c3" } },
                        {  "duck": { "id": "c4", "serverUrl": "http://localhost/c4" } }
                    ],
                    "views": [
                        {
                            "id": "foo",
                            "name": "Foo",
                            "collectors": [ "a1", "a2" ]
                        },
                        {
                            "id": "bar",
                            "name": "Bar",
                            "collectors": [ "b1", "b2", "b3" ]
                        },
                        {
                            "id": "baz",
                            "name": "Bar",
                            "collectors": [ "c1", "c2", "c3", "c4" ]
                        }
                    ]
                }"#,
            )
            .unwrap()
            .views
            .unwrap(),
        );

        // When
        let collectors = repository.get_collectors("bar").unwrap();

        // Then
        assert_eq!(3, collectors.len());
        assert!(collectors.contains("b1"));
        assert!(collectors.contains("b2"));
        assert!(collectors.contains("b3"));
    }
}
