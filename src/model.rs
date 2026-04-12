use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Spec,
    Group,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SourceLocation {
    pub path: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct CommentBlock {
    pub path: PathBuf,
    pub lines: Vec<BlockLine>,
}

#[derive(Debug, Clone)]
pub struct BlockLine {
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecDecl {
    pub id: String,
    pub kind: NodeKind,
    pub text: String,
    pub planned: bool,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerifyRef {
    pub spec_id: String,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize)]
pub struct AttestRef {
    pub spec_id: String,
    pub artifact: String,
    pub owner: String,
    pub last_reviewed: String,
    pub review_interval_days: Option<u32>,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub path: PathBuf,
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Default, Clone)]
pub struct ParsedRepo {
    pub specs: Vec<SpecDecl>,
    pub verifies: Vec<VerifyRef>,
    pub attests: Vec<AttestRef>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecNode {
    pub id: String,
    pub kind: NodeKind,
    pub text: String,
    pub planned: bool,
    pub location: SourceLocation,
    pub verifies: Vec<VerifyRef>,
    pub attests: Vec<AttestRef>,
    pub children: Vec<SpecNode>,
}

impl SpecNode {
    pub fn is_unsupported(&self) -> bool {
        self.kind == NodeKind::Spec
            && !self.planned
            && self.verifies.is_empty()
            && self.attests.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct SpecFilter {
    pub include_planned: bool,
    pub unsupported_only: bool,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecDocument {
    pub nodes: Vec<SpecNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LintReport {
    pub diagnostics: Vec<Diagnostic>,
}
