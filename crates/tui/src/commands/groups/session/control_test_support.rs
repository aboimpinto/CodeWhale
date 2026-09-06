//! FEAT-024 Phase 4: deterministic fake session-control facet for portable
//! handler tests. Canned returns plus a call log prove exact strings, actions,
//! call arguments, operation counts, and check order without touching the host.

#![cfg(test)]

use codewhale_command_contract::facets::{
    CommandSessionControlContext, HostedWorkTarget, PlanProjection, RelayProjection, RemoteLink,
    RemoteOpenOutcome, RemoteStartInfo, ResumeImportReceipt, ResumeSource, SessionTitleReceipt,
    TitleReport, TitleSetOutcome, TodoProjection,
};
use std::path::PathBuf;

#[derive(Default)]
pub(crate) struct FakeControl {
    pub(crate) blocked: bool,
    pub(crate) relay: Option<RelayProjection>,
    pub(crate) resume: Option<Result<ResumeSource, String>>,
    pub(crate) import: Option<Result<ResumeImportReceipt, String>>,
    pub(crate) rename: Option<Result<SessionTitleReceipt, String>>,
    pub(crate) title_report: Option<TitleReport>,
    pub(crate) set_title: Option<Result<TitleSetOutcome, String>>,
    pub(crate) remote_status: Option<String>,
    pub(crate) remote_link: Option<Option<RemoteLink>>,
    pub(crate) browser_open: Option<RemoteOpenOutcome>,
    pub(crate) start_info: Option<RemoteStartInfo>,
    pub(crate) stop_refusal: Option<Option<String>>,
    pub(crate) hosted: Option<Option<HostedWorkTarget>>,
    pub(crate) calls: Vec<String>,
}

impl FakeControl {
    fn call(&mut self, name: &str, arg: Option<&str>) {
        match arg {
            Some(arg) => self.calls.push(format!("{name}({arg})")),
            None => self.calls.push(name.to_string()),
        }
    }
}

impl CommandSessionControlContext for FakeControl {
    fn transition_blocked(&self) -> bool {
        self.blocked
    }
    fn relay_projection(&self) -> RelayProjection {
        self.relay.clone().expect("unexpected relay_projection()")
    }
    fn open_resume_picker(&mut self) {
        self.calls.push("open_resume_picker".to_string());
    }
    fn resolve_resume_source(&mut self, raw: &str) -> Result<ResumeSource, String> {
        self.call("resolve_resume_source", Some(raw));
        match self.resume.clone() {
            Some(result) => result,
            None => Ok(ResumeSource::NotFound {
                raw: raw.to_string(),
                error: "missing".to_string(),
            }),
        }
    }
    fn import_session_file(&mut self, path: PathBuf) -> Result<ResumeImportReceipt, String> {
        self.call("import_session_file", Some(&path.display().to_string()));
        match self.import.clone() {
            Some(result) => result,
            None => Err(format!(
                "unexpected import_session_file({}) on empty fake",
                path.display()
            )),
        }
    }
    fn rename_session(&mut self, raw: &str) -> Result<SessionTitleReceipt, String> {
        self.call("rename_session", Some(raw));
        match self.rename.clone() {
            Some(result) => result,
            None => Err(format!("unexpected rename_session({raw}) on empty fake")),
        }
    }
    fn title_report(&self) -> TitleReport {
        self.title_report
            .clone()
            .expect("unexpected title_report()")
    }
    fn set_window_title(&mut self, title: Option<String>) -> Result<TitleSetOutcome, String> {
        match &title {
            Some(value) => self.call("set_window_title", Some(value)),
            None => self.calls.push("set_window_title(clear)".to_string()),
        }
        match self.set_title.clone() {
            Some(result) => result,
            None => Err("unexpected set_window_title on empty fake".to_string()),
        }
    }
    fn remote_status(&self) -> String {
        self.remote_status
            .clone()
            .expect("unexpected remote_status()")
    }
    fn remote_link(&self) -> Option<RemoteLink> {
        self.remote_link.clone().expect("unexpected remote_link()")
    }
    fn remote_browser_open(&self) -> RemoteOpenOutcome {
        self.browser_open
            .clone()
            .expect("unexpected browser_open()")
    }
    fn remote_start_info(&self) -> RemoteStartInfo {
        self.start_info.clone().expect("unexpected start_info()")
    }
    fn remote_stop_refusal(&self) -> Option<String> {
        self.stop_refusal
            .clone()
            .expect("unexpected stop_refusal()")
    }
    fn resolve_hosted_work_target(&self) -> Option<HostedWorkTarget> {
        self.hosted.clone().expect("unexpected hosted()")
    }
}

pub(crate) fn relay_projection_fixture() -> RelayProjection {
    RelayProjection {
        compact_template: "# Session relay".to_string(),
        workspace: "/work".to_string(),
        mode: "operate".to_string(),
        model: "model-x".to_string(),
        goal_objective: Some("objective-y".to_string()),
        goal_token_budget: Some(900),
        todos: TodoProjection::Absent,
        plan: PlanProjection::Absent,
    }
}
