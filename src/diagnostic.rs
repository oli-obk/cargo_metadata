//! This module contains `Diagnostic` and the types/functions it uses for deserialization.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCode {
    /// The code itself.
    pub code: String,
    /// An explanation for the code
    pub explanation: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSpanLine {
    pub text: String,
    /// 1-based, character offset in self.text
    pub highlight_start: usize,
    pub highlight_end: usize
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSpanMacroExpansion {
    /// span where macro was applied to generate this code; note that
    /// this may itself derive from a macro (if
    /// `span.expansion.is_some()`)
    pub span: DiagnosticSpan,

    /// name of macro that was applied (e.g., "foo!" or "#[derive(Eq)]")
    pub macro_decl_name: String,

    /// span where macro was defined (if known)
    pub def_site_span: Option<DiagnosticSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSpan {
    pub file_name: String,
    pub byte_start: u32,
    pub byte_end: u32,
    /// 1-based.
    pub line_start: usize,
    pub line_end: usize,
    /// 1-based, character offset.
    pub column_start: usize,
    pub column_end: usize,
    /// Is this a "primary" span -- meaning the point, or one of the points,
    /// where the error occurred?
    pub is_primary: bool,
    /// Source text from the start of line_start to the end of line_end.
    pub text: Vec<DiagnosticSpanLine>,
    /// Label that should be placed at this location (if any)
    pub label: Option<String>,
    /// If we are suggesting a replacement, this will contain text
    /// that should be sliced in atop this span.
    pub suggested_replacement: Option<String>,
    /// If the suggestion is approximate
    pub suggestion_applicability: Option<Applicability>,
    /// Macro invocations that created the code at this span, if any.
    pub expansion: Option<Box<DiagnosticSpanMacroExpansion>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Applicability {
    MachineApplicable,
    HasPlaceholders,
    MaybeIncorrect,
    Unspecified
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    pub code: Option<DiagnosticCode>,
    /// "error: internal compiler error", "error", "warning", "note", "help"
    pub level: String,
    pub spans: Vec<DiagnosticSpan>,
    /// Associated diagnostic messages.
    pub children: Vec<Diagnostic>,
    /// The message as rustc would render it
    pub rendered: Option<String>
}

