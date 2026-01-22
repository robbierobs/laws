//! Pagination helpers for AWS SDK operations
//!
//! Provides macros to reduce boilerplate in paginated AWS API calls.

/// Macro to paginate through AWS API calls with next_token
///
/// # Example
/// ```ignore
/// let secrets = paginate_with_token!(
///     token,
///     {
///         let mut request = self.client.list_secrets();
///         if let Some(t) = token {
///             request = request.next_token(t);
///         }
///         request.send().await
///             .map_err(|e| format_sdk_error("SecretsManager", "list_secrets", "all", e))?
///     },
///     |response| {
///         response.secret_list().iter().map(|s| Secret::from_aws(s)).collect()
///     }
/// );
/// ```
#[macro_export]
macro_rules! paginate_with_token {
    ($token_var:ident, $request_block:block, $extract_fn:expr) => {{
        let mut all_items = Vec::new();
        let mut $token_var: Option<String> = None;

        loop {
            let response = $request_block;

            let items: Vec<_> = $extract_fn(&response);
            all_items.extend(items);

            $token_var = response.next_token().map(|s| s.to_string());
            if $token_var.is_none() {
                break;
            }
        }

        all_items
    }};
}

/// Macro to paginate through AWS API calls with marker (IAM, Lambda style)
///
/// # Example
/// ```ignore
/// let users = paginate_with_marker!(
///     marker,
///     {
///         let mut request = self.client.list_users();
///         if let Some(m) = marker {
///             request = request.marker(m);
///         }
///         request.send().await
///             .map_err(|e| format_sdk_error("IAM", "list_users", "all", e))?
///     },
///     |response| {
///         response.users().iter().map(|u| IamUser::from_aws(u)).collect()
///     },
///     |response| response.is_truncated()
/// );
/// ```
#[macro_export]
macro_rules! paginate_with_marker {
    ($marker_var:ident, $request_block:block, $extract_fn:expr, $is_truncated_fn:expr) => {{
        let mut all_items = Vec::new();
        let mut $marker_var: Option<String> = None;

        loop {
            let response = $request_block;

            let items: Vec<_> = $extract_fn(&response);
            all_items.extend(items);

            let is_truncated = $is_truncated_fn(&response);
            $marker_var = response.marker().map(|s| s.to_string());
            if $marker_var.is_none() || !is_truncated {
                break;
            }
        }

        all_items
    }};
}

/// Macro to paginate through AWS API calls with continuation_token (S3 style)
///
/// # Example
/// ```ignore
/// let objects = paginate_with_continuation!(
///     token,
///     {
///         let mut request = self.client.list_objects_v2().bucket(bucket_name);
///         if let Some(t) = token {
///             request = request.continuation_token(t);
///         }
///         request.send().await
///             .map_err(|e| format_s3_error("list_objects_v2", bucket_name, e))?
///     },
///     |response| {
///         response.contents().iter().map(|obj| S3Object::from_aws(obj, bucket_name)).collect()
///     }
/// );
/// ```
#[macro_export]
macro_rules! paginate_with_continuation {
    ($token_var:ident, $request_block:block, $extract_fn:expr) => {{
        let mut all_items = Vec::new();
        let mut $token_var: Option<String> = None;

        loop {
            let response = $request_block;

            let items: Vec<_> = $extract_fn(&response);
            all_items.extend(items);

            $token_var = response.next_continuation_token().map(|s| s.to_string());
            if $token_var.is_none() {
                break;
            }
        }

        all_items
    }};
}

#[cfg(test)]
mod tests {
    // Note: These macros are tested via integration in the AWS service modules
    // Unit testing macros directly is complex and not idiomatic in Rust
}
