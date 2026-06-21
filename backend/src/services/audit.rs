//! Audit service — record security and administrative events.
//! No SQL here. All DB access goes through repositories.

use serde_json::Value;
use uuid::Uuid;

use crate::repositories::{AuditRepository, audit::parse_user_agent};
use crate::state::AppState;

/// Record an audit event. Fire-and-forget: errors are logged but never propagated.
#[allow(clippy::too_many_arguments)]
pub async fn record(
    state: &AppState,
    user_id: Option<Uuid>,
    user_email: Option<&str>,
    action: &str,
    resource: &str,
    resource_id: Option<&str>,
    ip: &str,
    user_agent: &str,
    success: bool,
    details: Option<Value>,
) {
    let ua_info = parse_user_agent(user_agent);
    let final_details = match details {
        Some(mut d) => {
            if let Some(obj) = d.as_object_mut() {
                obj.insert("user_agent_info".to_string(), ua_info);
            }
            d
        }
        None => ua_info,
    };

    let _ = AuditRepository::create(
        &state.db.pool,
        user_id,
        user_email,
        action,
        resource,
        resource_id,
        ip,
        user_agent,
        success,
        Some(&final_details),
    )
    .await;
}
