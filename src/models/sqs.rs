use aws_sdk_sqs::types::QueueAttributeName;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqsQueue {
    pub queue_url: String,
    pub queue_name: String,
    pub queue_arn: Option<String>,
    pub approximate_number_of_messages: i64,
    pub approximate_number_of_messages_not_visible: i64,
    pub approximate_number_of_messages_delayed: i64,
    pub visibility_timeout: i32,
    pub maximum_message_size: i32,
    pub message_retention_period: i32,
    pub delay_seconds: i32,
    pub receive_message_wait_time_seconds: i32,
    pub created_timestamp: Option<String>,
    pub last_modified_timestamp: Option<String>,
    pub is_fifo: bool,
    pub content_based_deduplication: bool,
    pub redrive_policy: Option<String>,
    pub dead_letter_target_arn: Option<String>,
}

impl SqsQueue {
    pub fn from_aws(queue_url: &str, attrs: Option<&HashMap<QueueAttributeName, String>>) -> Self {
        let queue_name = queue_url
            .rsplit('/')
            .next()
            .unwrap_or("unknown")
            .to_string();

        let is_fifo = queue_name.ends_with(".fifo");

        let get_attr =
            |name: QueueAttributeName| -> Option<&String> { attrs.and_then(|a| a.get(&name)) };

        let redrive_policy = get_attr(QueueAttributeName::RedrivePolicy).cloned();
        let dead_letter_target_arn = redrive_policy.as_ref().and_then(|rp| {
            serde_json::from_str::<serde_json::Value>(rp)
                .ok()
                .and_then(|v| {
                    v.get("deadLetterTargetArn")
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string())
                })
        });

        Self {
            queue_url: queue_url.to_string(),
            queue_name,
            queue_arn: get_attr(QueueAttributeName::QueueArn).cloned(),
            approximate_number_of_messages: get_attr(
                QueueAttributeName::ApproximateNumberOfMessages,
            )
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
            approximate_number_of_messages_not_visible: get_attr(
                QueueAttributeName::ApproximateNumberOfMessagesNotVisible,
            )
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
            approximate_number_of_messages_delayed: get_attr(
                QueueAttributeName::ApproximateNumberOfMessagesDelayed,
            )
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
            visibility_timeout: get_attr(QueueAttributeName::VisibilityTimeout)
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            maximum_message_size: get_attr(QueueAttributeName::MaximumMessageSize)
                .and_then(|s| s.parse().ok())
                .unwrap_or(262144),
            message_retention_period: get_attr(QueueAttributeName::MessageRetentionPeriod)
                .and_then(|s| s.parse().ok())
                .unwrap_or(345600),
            delay_seconds: get_attr(QueueAttributeName::DelaySeconds)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            receive_message_wait_time_seconds: get_attr(
                QueueAttributeName::ReceiveMessageWaitTimeSeconds,
            )
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
            created_timestamp: get_attr(QueueAttributeName::CreatedTimestamp).cloned(),
            last_modified_timestamp: get_attr(QueueAttributeName::LastModifiedTimestamp).cloned(),
            is_fifo,
            content_based_deduplication: get_attr(QueueAttributeName::ContentBasedDeduplication)
                .map(|s| s == "true")
                .unwrap_or(false),
            redrive_policy,
            dead_letter_target_arn,
        }
    }

    pub fn total_messages(&self) -> i64 {
        self.approximate_number_of_messages
            + self.approximate_number_of_messages_not_visible
            + self.approximate_number_of_messages_delayed
    }

    pub fn format_retention(&self) -> String {
        let days = self.message_retention_period / 86400;
        if days >= 1 {
            format!("{} days", days)
        } else {
            let hours = self.message_retention_period / 3600;
            format!("{} hours", hours)
        }
    }

    pub fn format_max_size(&self) -> String {
        let kb = self.maximum_message_size / 1024;
        format!("{} KB", kb)
    }

    pub fn queue_type(&self) -> &'static str {
        if self.is_fifo {
            "FIFO"
        } else {
            "Standard"
        }
    }

    pub fn has_dlq(&self) -> bool {
        self.dead_letter_target_arn.is_some()
    }
}

impl crate::models::Filterable for SqsQueue {
    fn matches_filter(&self, filter: &str) -> bool {
        self.queue_name.to_lowercase().contains(filter)
            || self.queue_url.to_lowercase().contains(filter)
    }
}
