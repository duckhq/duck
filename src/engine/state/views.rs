use std::collections::HashMap;
use std::collections::HashSet;

use crate::config::{Configuration, ViewConfiguration};

pub struct ViewRepository {
    views: Vec<ViewConfiguration>,
    collectors: HashMap<String, HashSet<String>>,
}

impl ViewRepository {
    pub fn new(config: &Configuration) -> Self {
        let mut map = HashMap::<String, HashSet<String>>::new();
        if let Some(views) = &config.views {
            for view in views.iter() {
                let mut collectors = HashSet::<String>::new();
                for collector in view.collectors.iter() {
                    collectors.insert(collector.clone());
                }
                map.insert(view.id.clone(), collectors);
            }
        }

        Self {
            collectors: map,
            views: match &config.views {
                Some(views) => views.clone(),
                None => vec![],
            },
        }
    }

    pub fn get_collectors(&self, view_id: &str) -> Option<&HashSet<String>> {
        self.collectors.get(view_id)
    }

    pub fn get_views(&self) -> &Vec<ViewConfiguration> {
        &self.views
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::text::TestVariableProvider;

    #[test]
    fn should_return_collectors_for_view() {
        // Given
        let views = ViewRepository::new(
            &Configuration::from_json(
                &TestVariableProvider::new(),
                r#"
            { 
                "collectors": [ ],
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
            }
        "#,
            )
            .unwrap(),
        );

        // When
        let collectors = views.get_collectors("bar").unwrap();

        // Then
        assert_eq!(3, collectors.len());
        assert!(collectors.contains("b1"));
        assert!(collectors.contains("b2"));
        assert!(collectors.contains("b3"));
    }
}
