//! Lambda-specific actions

/// Lambda-specific actions
#[derive(Debug, Clone)]
pub enum LambdaAction {
    InvokeFunction(String),
    DeleteFunction(String),
    LoadFunctionDetails(String),
}
