//! ECS service update handlers
//!
//! These handlers have multiple parameters that mirror the EcsAction
//! enum structure and are called from a single dispatch point.

#![allow(clippy::too_many_arguments)]

use crate::app::task_manager::task_keys;
use crate::app::{App, EventSender};
use crate::event::{AwsEvent, Event};

impl App {
    pub fn handle_ecs_update_desired_count(
        &mut self,
        cluster_arn: String,
        service_name: String,
        desired_count: i32,
        event_tx: EventSender,
    ) {
        let cluster = cluster_arn.clone();
        let service = service_name.clone();

        self.spawn_aws_task(
            event_tx,
            task_keys::ECS_ACTION,
            move |clients, tx| async move {
                let ecs_client = crate::aws::ecs::EcsClient::new(clients.ecs.clone());
                match ecs_client
                    .update_service_desired_count(&cluster, &service, desired_count)
                    .await
                {
                    Ok(()) => {
                        tx.send(Event::Aws(AwsEvent::ActionCompleted(format!(
                            "Updated {} desired count to {}",
                            service, desired_count
                        ))))
                        .await
                        .ok();

                        // Refresh services list
                        match ecs_client.list_services(&cluster).await {
                            Ok(services) => {
                                tx.send(Event::Aws(AwsEvent::EcsServicesLoaded(services)))
                                    .await
                                    .ok();
                            }
                            Err(e) => {
                                tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                    .await
                                    .ok();
                            }
                        }
                    }
                    Err(e) => {
                        tx.send(Event::Aws(AwsEvent::Error(format!(
                            "Failed to update {}: {}",
                            service, e
                        ))))
                        .await
                        .ok();
                    }
                }
            }
        );
    }

    pub fn handle_ecs_force_new_deployment(
        &mut self,
        cluster_arn: String,
        service_name: String,
        event_tx: EventSender,
    ) {
        let cluster = cluster_arn.clone();
        let service = service_name.clone();

        self.spawn_aws_task(
            event_tx,
            task_keys::ECS_ACTION,
            move |clients, tx| async move {
                let ecs_client = crate::aws::ecs::EcsClient::new(clients.ecs.clone());
                match ecs_client
                    .force_new_deployment(&cluster, &service)
                    .await
                {
                    Ok(()) => {
                        tx.send(Event::Aws(AwsEvent::ActionCompleted(format!(
                            "Forced new deployment for {}",
                            service
                        ))))
                        .await
                        .ok();

                        // Refresh services list
                        match ecs_client.list_services(&cluster).await {
                            Ok(services) => {
                                tx.send(Event::Aws(AwsEvent::EcsServicesLoaded(services)))
                                    .await
                                    .ok();
                            }
                            Err(e) => {
                                tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                    .await
                                    .ok();
                            }
                        }
                    }
                    Err(e) => {
                        tx.send(Event::Aws(AwsEvent::Error(format!(
                            "Failed to force deployment for {}: {}",
                            service, e
                        ))))
                        .await
                        .ok();
                    }
                }
            }
        );
    }

    /// Handle updating an ECS service with optional task definition, CPU, and memory changes
    pub fn handle_ecs_update_service(
        &mut self,
        cluster_arn: String,
        service_name: String,
        task_definition: Option<String>,
        cpu: Option<String>,
        memory: Option<String>,
        force_new_deployment: bool,
        event_tx: EventSender,
    ) {
        let cluster = cluster_arn.clone();
        let service = service_name.clone();
        // Clone options for closure
        let td = task_definition.clone();
        let cpu_val = cpu.clone();
        let mem_val = memory.clone();

        self.spawn_aws_task(
            event_tx,
            task_keys::ECS_ACTION,
            move |clients, tx| async move {
                let ecs_client = crate::aws::ecs::EcsClient::new(clients.ecs.clone());
                
                // Build description for action completion message
                let mut changes = Vec::new();
                if td.is_some() {
                    changes.push("task definition".to_string());
                }
                if let Some(c) = &cpu_val {
                    changes.push(format!("CPU to {}", c));
                }
                if let Some(m) = &mem_val {
                    changes.push(format!("memory to {}", m));
                }
                
                match ecs_client
                    .update_service(
                        &cluster,
                        &service,
                        td.as_deref(),
                        cpu_val.as_deref(),
                        mem_val.as_deref(),
                        force_new_deployment,
                    )
                    .await
                {
                    Ok(new_task_def_arn) => {
                        let short_arn = new_task_def_arn.split('/').next_back().unwrap_or(&new_task_def_arn);
                        tx.send(Event::Aws(AwsEvent::ActionCompleted(format!(
                            "Updated {} - {} (using {})",
                            service,
                            changes.join(", "),
                            short_arn
                        ))))
                        .await
                        .ok();

                        // Refresh services list
                        match ecs_client.list_services(&cluster).await {
                            Ok(services) => {
                                tx.send(Event::Aws(AwsEvent::EcsServicesLoaded(services)))
                                    .await
                                    .ok();
                            }
                            Err(e) => {
                                tx.send(Event::Aws(AwsEvent::Error(e.to_string())))
                                    .await
                                    .ok();
                            }
                        }
                    }
                    Err(e) => {
                        tx.send(Event::Aws(AwsEvent::Error(format!(
                            "Failed to update {}: {}",
                            service, e
                        ))))
                        .await
                        .ok();
                    }
                }
            }
        );
    }
}
