pub mod backup;
pub mod billing;
pub mod budgets;
pub mod cloudtrail;
pub mod dynamodb;
pub mod ec2;
pub mod ecr;
pub mod ecs;
pub mod iam;
pub mod ids;
pub mod lambda;
pub mod rds;
pub mod s3;
pub mod secretsmanager;
pub mod sqs;
pub mod vpc;

use ratatui::style::Color;

pub trait Filterable {
    fn matches_filter(&self, filter: &str) -> bool;
}

pub trait StateColor {
    fn state_color(&self) -> Color;
}

pub trait TagView {
    fn key(&self) -> Option<&str>;
    fn value(&self) -> Option<&str>;
}

pub fn collect_tags<'a, T>(tags: impl IntoIterator<Item = &'a T>) -> Vec<(String, String)>
where
    T: TagView + 'a,
{
    let mut collected: Vec<(String, String)> = tags
        .into_iter()
        .filter_map(|tag| match (tag.key(), tag.value()) {
            (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
            _ => None,
        })
        .collect();

    collected.sort_by(|a, b| a.0.cmp(&b.0));
    collected
}

pub fn find_name_tag<'a, T>(tags: impl IntoIterator<Item = &'a T>) -> Option<String>
where
    T: TagView + 'a,
{
    tags.into_iter()
        .find(|tag| tag.key() == Some("Name"))
        .and_then(|tag| tag.value().map(|value| value.to_string()))
}

impl TagView for aws_sdk_ec2::types::Tag {
    fn key(&self) -> Option<&str> {
        self.key()
    }

    fn value(&self) -> Option<&str> {
        self.value()
    }
}

impl TagView for aws_sdk_rds::types::Tag {
    fn key(&self) -> Option<&str> {
        self.key()
    }

    fn value(&self) -> Option<&str> {
        self.value()
    }
}

impl TagView for aws_sdk_ecs::types::Tag {
    fn key(&self) -> Option<&str> {
        self.key()
    }

    fn value(&self) -> Option<&str> {
        self.value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestTag {
        key: Option<&'static str>,
        value: Option<&'static str>,
    }

    impl TagView for TestTag {
        fn key(&self) -> Option<&str> {
            self.key
        }

        fn value(&self) -> Option<&str> {
            self.value
        }
    }

    #[test]
    fn test_collect_tags_filters_missing_and_sorts() {
        let tags = vec![
            TestTag {
                key: Some("Env"),
                value: Some("Prod"),
            },
            TestTag {
                key: None,
                value: Some("NoKey"),
            },
            TestTag {
                key: Some("Name"),
                value: Some("App"),
            },
            TestTag {
                key: Some("Owner"),
                value: None,
            },
        ];

        let collected = collect_tags(tags.iter());

        assert_eq!(
            collected,
            vec![
                ("Env".to_string(), "Prod".to_string()),
                ("Name".to_string(), "App".to_string())
            ]
        );
    }

    #[test]
    fn test_find_name_tag_returns_value() {
        let tags = vec![
            TestTag {
                key: Some("Env"),
                value: Some("Dev"),
            },
            TestTag {
                key: Some("Name"),
                value: Some("Service"),
            },
        ];

        let name = find_name_tag(tags.iter());

        assert_eq!(name, Some("Service".to_string()));
    }

    #[test]
    fn test_find_name_tag_missing_returns_none() {
        let tags = vec![TestTag {
            key: Some("Env"),
            value: Some("Prod"),
        }];

        let name = find_name_tag(tags.iter());

        assert_eq!(name, None);
    }
}
