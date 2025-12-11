warning: unused import: `service_state::ServiceStates`
  --> src/app/mod.rs:30:9
   |
30 | pub use service_state::ServiceStates;
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused import: `task_manager::TaskManager`
  --> src/app/mod.rs:32:9
   |
32 | pub use task_manager::TaskManager;
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `filtered_list::FilteredList`
  --> src/app/mod.rs:34:9
   |
34 | pub use filtered_list::FilteredList;
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: method `to_index` is never used
   --> src/app/messages.rs:198:12
    |
197 | impl DynamoDbViewMode {
    | --------------------- method in this implementation
198 |     pub fn to_index(self) -> usize {
    |            ^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: variants `Lambda`, `Backup`, and `CloudTrail` are never constructed
   --> src/app/messages.rs:312:5
    |
307 | pub enum ServiceAction {
    |          ------------- variants in this enum
...
312 |     Lambda(LambdaAction),
    |     ^^^^^^
...
315 |     Backup(BackupAction),
    |     ^^^^^^
316 |     CloudTrail(CloudTrailAction),
    |     ^^^^^^^^^^
    |
    = note: `ServiceAction` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: associated function `new` is never used
  --> src/config.rs:88:12
   |
86 | impl AppConfig {
   | -------------- associated function in this implementation
87 |     /// Create a new AppConfig with default values
88 |     pub fn new() -> Self {
   |            ^^^

warning: method `preview` is never used
  --> src/models/dynamodb.rs:51:12
   |
44 | impl DynamoDbItem {
   | ----------------- method in this implementation
...
51 |     pub fn preview(&self, max_attrs: usize) -> String {
   |            ^^^^^^^