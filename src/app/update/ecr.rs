use crate::app::messages::ExportFormat;
use crate::app::states::ecr::EcrImageFilters;
use crate::app::task_manager::task_keys;
use crate::app::{
    messages::{EcrAction, EcrViewMode},
    App,
};
use crate::event::{AwsEvent, Event};

impl App {
    pub fn handle_ecr_action(&mut self, action: EcrAction, event_tx: crate::app::EventSender) {
        match action {
            EcrAction::LoadImages(repo_name) => {
                self.services.ecr.selected_repo_name = Some(repo_name.clone());
                self.services.ecr.view_mode = EcrViewMode::Images;
                self.services.ecr.images.clear();
                self.services.ecr.list_state.select(None);
                self.loading = true;

                self.load_ecr_images(repo_name, None, false, event_tx);
            }
            EcrAction::BackToRepositories => {
                self.services.ecr.view_mode = EcrViewMode::Repositories;
                self.services.ecr.selected_repo_name = None;
                self.services.ecr.images.clear();
                self.services.ecr.list_state.select(None);
                // Reset filter
                self.services.ecr.filter_modal.clear_all();
                self.services.ecr.current_filters = EcrImageFilters::default();
            }
            EcrAction::PullImage {
                repository_uri,
                image_tag,
            } => {
                self.handle_pull_image(repository_uri, image_tag, event_tx);
            }
            EcrAction::LoadMoreImages => {
                if let Some(repo_name) = self.services.ecr.selected_repo_name.clone() {
                    if let Some(token) = self.services.ecr.images.next_token.clone() {
                        self.services.ecr.images.start_loading_more();
                        self.load_ecr_images(repo_name, Some(token), true, event_tx);
                    }
                }
            }
            EcrAction::OpenFilterModal => {
                self.services.ecr.filter_modal.open();
                self.input_mode = crate::app::InputMode::EcrImageFilter;
            }
            EcrAction::CloseFilterModal => {
                self.services.ecr.filter_modal.close();
                self.input_mode = crate::app::InputMode::Normal;
            }
            EcrAction::ApplyFilters => {
                // Update current filters from modal state
                self.services.ecr.current_filters =
                    EcrImageFilters::from_modal_state(&self.services.ecr.filter_modal);
                self.services.ecr.filter_modal.close();
                self.input_mode = crate::app::InputMode::Normal;

                // Reload images with new filter
                if let Some(repo_name) = self.services.ecr.selected_repo_name.clone() {
                    self.services.ecr.images.clear();
                    self.loading = true;
                    self.load_ecr_images(repo_name, None, false, event_tx);
                }
            }
            EcrAction::ClearFilters => {
                // Clear all filters
                self.services.ecr.filter_modal.clear_all();
                self.services.ecr.current_filters = EcrImageFilters::default();

                // Reload images without filters
                if let Some(repo_name) = self.services.ecr.selected_repo_name.clone() {
                    self.services.ecr.images.clear();
                    self.loading = true;
                    self.load_ecr_images(repo_name, None, false, event_tx);
                }
            }
            EcrAction::LoadScanFindings {
                repository_name,
                image_digest,
            } => {
                self.load_ecr_scan_findings(repository_name, image_digest, event_tx);
            }
            EcrAction::ExportScanFindings { format } => {
                self.export_ecr_scan_findings(format, event_tx);
            }
        }
    }

    /// Load ECR images with pagination support
    fn load_ecr_images(
        &mut self,
        repo_name: String,
        next_token: Option<String>,
        append: bool,
        event_tx: crate::app::EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        let client = clients.ecr.clone();
        let tx = event_tx.clone();
        let tag_filter = self
            .services
            .ecr
            .current_filters
            .tag_status_api_value()
            .map(|s| s.to_string());
        let max_results = self.config.max_ecr_images as i32;

        let handle = tokio::spawn(async move {
            let ecr_service = crate::aws::ecr::EcrService::new(client);
            match ecr_service
                .describe_images_paginated(
                    &repo_name,
                    Some(max_results),
                    next_token.as_deref(),
                    tag_filter.as_deref(),
                )
                .await
            {
                Ok(result) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::EcrImagesLoaded {
                        images: result.images,
                        next_token: result.next_token,
                        append,
                    })))
                    .await
                    .ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(e.to_string()))))
                        .await
                        .ok();
                }
            }
        });
        self.tasks.spawn(task_keys::ECR_IMAGES, handle);
    }

    /// Load scan findings for a specific image
    fn load_ecr_scan_findings(
        &mut self,
        repository_name: String,
        image_digest: String,
        event_tx: crate::app::EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        // Skip if already loaded for this digest
        if self.services.ecr.scan_findings_digest.as_ref() == Some(&image_digest) {
            return;
        }

        self.services.ecr.scan_findings_loading = true;
        self.services.ecr.scan_findings = None;

        let client = clients.ecr.clone();
        let tx = event_tx.clone();
        let digest_clone = image_digest.clone();

        let handle = tokio::spawn(async move {
            let ecr_service = crate::aws::ecr::EcrService::new(client);
            match ecr_service
                .describe_image_scan_findings(&repository_name, &digest_clone)
                .await
            {
                Ok(findings) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::EcrScanFindingsLoaded {
                        image_digest: digest_clone,
                        findings,
                    })))
                    .await
                    .ok();
                }
                Err(e) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                        "Failed to load scan findings: {}",
                        e
                    )))))
                    .await
                    .ok();
                }
            }
        });
        self.tasks.spawn(task_keys::ECR_SCAN_FINDINGS, handle);
    }

    /// Export scan findings to a file (JSON or CSV)
    fn export_ecr_scan_findings(
        &mut self,
        format: ExportFormat,
        event_tx: crate::app::EventSender,
    ) {
        let Some(findings) = &self.services.ecr.scan_findings else {
            self.error_message = Some("No scan findings loaded. Press 'v' to load first.".into());
            return;
        };

        let Some(digest) = &self.services.ecr.scan_findings_digest else {
            return;
        };

        // Build filename
        let short_digest = digest.chars().skip(7).take(12).collect::<String>();
        let extension = match format {
            ExportFormat::Json => "json",
            ExportFormat::Csv => "csv",
        };
        let filename = format!(
            "ecr_scan_{}_{}.{}",
            short_digest,
            chrono::Utc::now().format("%Y%m%d_%H%M%S"),
            extension
        );

        // Get downloads directory or fall back to temp
        let downloads_dir = dirs::download_dir().unwrap_or_else(std::env::temp_dir);
        let file_path = downloads_dir.join(&filename);

        let content = match format {
            ExportFormat::Json => match serde_json::to_string_pretty(findings) {
                Ok(json) => json,
                Err(e) => {
                    self.error_message = Some(format!("Failed to serialize JSON: {}", e));
                    return;
                }
            },
            ExportFormat::Csv => {
                let mut csv =
                    String::from("severity,name,description,package,version,fixed_version\n");

                // Enhanced findings
                for f in &findings.enhanced_findings {
                    let severity = f.severity.as_deref().unwrap_or("");
                    let name = f.title.as_deref().unwrap_or("").replace('"', "\"\"");
                    let desc = f
                        .description
                        .as_deref()
                        .unwrap_or("")
                        .replace('"', "\"\"")
                        .chars()
                        .take(200)
                        .collect::<String>();
                    let pkg = f.package_name.as_deref().unwrap_or("");
                    let ver = f.package_version.as_deref().unwrap_or("");
                    let fixed = f.fixed_version.as_deref().unwrap_or("");
                    csv.push_str(&format!(
                        "{},\"{}\",\"{}\",{},{},{}\n",
                        severity, name, desc, pkg, ver, fixed
                    ));
                }

                // Basic findings
                for f in &findings.findings {
                    let severity = f.severity.as_deref().unwrap_or("");
                    let name = f.name.as_deref().unwrap_or("").replace('"', "\"\"");
                    let desc = f
                        .description
                        .as_deref()
                        .unwrap_or("")
                        .replace('"', "\"\"")
                        .chars()
                        .take(200)
                        .collect::<String>();
                    csv.push_str(&format!("{},\"{}\",\"{}\",,,\n", severity, name, desc));
                }

                csv
            }
        };

        let path_str = file_path.to_string_lossy().to_string();
        let tx = event_tx.clone();

        // Write synchronously since it's small
        match std::fs::write(&file_path, content) {
            Ok(_) => {
                let path_clone = path_str.clone();
                tokio::spawn(async move {
                    tx.send(Event::Aws(Box::new(AwsEvent::EcrScanFindingsExported {
                        path: path_clone,
                    })))
                    .await
                    .ok();
                });
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to write file: {}", e));
            }
        }
    }

    /// Pull an ECR image by:
    /// 1. Getting ECR authorization token
    /// 2. Running docker login
    /// 3. Running docker pull
    fn handle_pull_image(
        &mut self,
        repository_uri: String,
        image_tag: String,
        event_tx: crate::app::EventSender,
    ) {
        let Some(clients) = &self.aws_clients else {
            return;
        };

        let image_uri = format!("{}:{}", repository_uri, image_tag);
        self.action_log
            .push(format!("Pulling image: {}", image_uri));
        self.loading = true;

        let client = clients.ecr.clone();
        let tx = event_tx.clone();
        let image_uri_clone = image_uri.clone();

        let handle = tokio::spawn(async move {
            let ecr_service = crate::aws::ecr::EcrService::new(client);

            // Step 1: Get authorization token
            let (username, password, proxy_endpoint) =
                match ecr_service.get_authorization_token().await {
                    Ok(auth) => auth,
                    Err(e) => {
                        tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                            "Failed to get ECR auth token: {}",
                            e
                        )))))
                        .await
                        .ok();
                        return;
                    }
                };

            // Step 2: Run docker login
            let login_result = tokio::process::Command::new("docker")
                .args([
                    "login",
                    "--username",
                    &username,
                    "--password-stdin",
                    &proxy_endpoint,
                ])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();

            match login_result {
                Ok(mut child) => {
                    // Write password to stdin
                    if let Some(mut stdin) = child.stdin.take() {
                        use tokio::io::AsyncWriteExt;
                        if let Err(e) = stdin.write_all(password.as_bytes()).await {
                            tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                                "Failed to write docker login password: {}",
                                e
                            )))))
                            .await
                            .ok();
                            return;
                        }
                        drop(stdin);
                    }

                    match child.wait_with_output().await {
                        Ok(output) => {
                            if !output.status.success() {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                                    "Docker login failed: {}",
                                    stderr.trim()
                                )))))
                                .await
                                .ok();
                                return;
                            }
                        }
                        Err(e) => {
                            tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                                "Docker login process failed: {}",
                                e
                            )))))
                            .await
                            .ok();
                            return;
                        }
                    }
                }
                Err(e) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                        "Failed to start docker login: {}. Is Docker installed and running?",
                        e
                    )))))
                    .await
                    .ok();
                    return;
                }
            }

            // Step 3: Run docker pull
            let pull_result = tokio::process::Command::new("docker")
                .args(["pull", &image_uri_clone])
                .output()
                .await;

            match pull_result {
                Ok(output) => {
                    if output.status.success() {
                        tx.send(Event::Aws(Box::new(AwsEvent::EcrImagePulled {
                            image_uri: image_uri_clone,
                        })))
                        .await
                        .ok();
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                            "Docker pull failed: {}",
                            stderr.trim()
                        )))))
                        .await
                        .ok();
                    }
                }
                Err(e) => {
                    tx.send(Event::Aws(Box::new(AwsEvent::Error(format!(
                        "Failed to run docker pull: {}",
                        e
                    )))))
                    .await
                    .ok();
                }
            }
        });

        self.tasks.spawn(task_keys::ECR_ACTION, handle);
    }
}
