//! FEAT-023 Phase 4/6 test support: a deterministic canned implementation of
//! `CommandSessionLifecycleContext` so portable handlers are unit-tested for
//! exact message/action composition without host state.

use codewhale_command_contract::facets::{
    CommandSessionLifecycleContext, SessionArchiveReceipt, SessionBranchOutcome,
    SessionForkReceipt, SessionNewReceipt, SessionSaveReceipt, SessionSyncPayload,
    TreeBodyProjection,
};
use std::path::PathBuf;

/// Every delegate returns the canned value set by the test; unimplemented
/// slots panic with a descriptive message so a test can never pass by accident.
pub(crate) struct CannedLifecycle {
    pub blocked: bool,
    pub leaf_hint: Option<String>,
    pub branch: Result<SessionBranchOutcome, String>,
    pub tree: Result<TreeBodyProjection, String>,
    pub save: Result<SessionSaveReceipt, String>,
    pub fork_active: Result<SessionForkReceipt, String>,
    pub fork_from: Result<SessionForkReceipt, String>,
    pub fresh: Result<SessionNewReceipt, String>,
    pub load: Result<PathBuf, String>,
    pub archived: Result<SessionArchiveReceipt, String>,
    pub prune: Result<usize, String>,
    pub picker: Option<String>,
}

impl Default for CannedLifecycle {
    fn default() -> Self {
        Self {
            blocked: false,
            leaf_hint: None,
            branch: Err("canned: branch_to not configured".to_string()),
            tree: Ok(TreeBodyProjection::NoSession),
            save: Err("canned: save not configured".to_string()),
            fork_active: Err("canned: fork_active not configured".to_string()),
            fork_from: Err("canned: fork_from not configured".to_string()),
            fresh: Err("canned: fresh_session not configured".to_string()),
            load: Err("canned: load not configured".to_string()),
            archived: Err("canned: set_archived not configured".to_string()),
            prune: Err("canned: prune not configured".to_string()),
            picker: None,
        }
    }
}

pub(crate) fn sync_payload(session_id: &str) -> SessionSyncPayload {
    SessionSyncPayload {
        session_id: Some(session_id.to_string()),
        messages: vec![],
        system_prompt: None,
        model: "test-model".to_string(),
        workspace: PathBuf::from("/workspace"),
        mode: codewhale_command_contract::types::CommandMode::Agent,
    }
}

impl CommandSessionLifecycleContext for CannedLifecycle {
    fn transition_blocked(&self) -> bool {
        self.blocked
    }
    fn branch_current_leaf_hint(&self) -> Option<String> {
        self.leaf_hint.clone()
    }
    fn branch_to(&mut self, _entry_id: &str) -> Result<SessionBranchOutcome, String> {
        self.branch.clone()
    }
    fn tree_body(&self) -> Result<TreeBodyProjection, String> {
        self.tree.clone()
    }
    fn save_session(&mut self, _p: Option<String>) -> Result<SessionSaveReceipt, String> {
        self.save.clone()
    }
    fn fork_active(&mut self) -> Result<SessionForkReceipt, String> {
        self.fork_active.clone()
    }
    fn fork_from(&mut self, _id: &str) -> Result<SessionForkReceipt, String> {
        self.fork_from.clone()
    }
    fn fresh_session(&mut self, _force: bool) -> Result<SessionNewReceipt, String> {
        self.fresh.clone()
    }
    fn load_session(&mut self, _path: &str) -> Result<PathBuf, String> {
        self.load.clone()
    }
    fn open_picker(&mut self, preselected: Option<String>) {
        self.picker = preselected;
    }
    fn set_archived(
        &mut self,
        _id: &str,
        _archived: bool,
    ) -> Result<SessionArchiveReceipt, String> {
        self.archived.clone()
    }
    fn prune_sessions(&mut self, _days: u64) -> Result<usize, String> {
        self.prune.clone()
    }
}
