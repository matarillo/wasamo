#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub filename: String,
    pub line: u32,
    pub col: u32,
}

impl Diagnostic {
    pub fn error(filename: &str, line: u32, col: u32, message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Error,
            message: message.into(),
            filename: filename.to_string(),
            line,
            col,
        }
    }

    pub fn warning(filename: &str, line: u32, col: u32, message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Warning,
            message: message.into(),
            filename: filename.to_string(),
            line,
            col,
        }
    }

    pub fn render(&self, src: &str) -> String {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        let header = format!("{}: {}", severity, self.message);
        let loc = format!("  --> {}:{}:{}", self.filename, self.line, self.col);

        let line_idx = self.line.saturating_sub(1) as usize;
        if let Some(line_text) = src.lines().nth(line_idx) {
            let line_num = self.line.to_string();
            let bar = " ".repeat(line_num.len());
            let indicator = format!("{}^", " ".repeat(self.col.saturating_sub(1) as usize));
            format!(
                "{}\n{}\n{} |\n{} | {}\n{} | {}",
                header, loc, bar, line_num, line_text, bar, indicator
            )
        } else {
            format!("{}\n{}", header, loc)
        }
    }
}
