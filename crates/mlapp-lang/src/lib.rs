//! MATLAB App Designer Language Frontend: classdef parser, AST extraction,
//! code extractor from XML containers, and static compatibility scanner.

use rustlab_parse::ast::Stmt;
use rustlab_parse::span::Span;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LangError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("XML extraction error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("Classdef definition not found in code")]
    MissingClassDef,

    #[error("Syntax error at line {line}, col {col}: {msg}")]
    Syntax {
        line: usize,
        col: usize,
        msg: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessModifier {
    Public,
    Private,
    Protected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppProperty {
    pub name: String,
    pub access: AccessModifier,
    pub type_annotation: Option<String>,
    pub initial_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MethodRole {
    StartupFcn,
    Callback { event_type: Option<String> },
    Helper,
}

#[derive(Debug, Clone)]
pub struct AppMethod {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub role: MethodRole,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AppClassDef {
    pub name: String,
    pub super_class: Option<String>,
    pub properties: Vec<AppProperty>,
    pub methods: HashMap<String, AppMethod>,
    pub raw_source: String,
}

/// Compatibility diagnostic report for static scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatReport {
    pub compatibility_score: u32, // 0 - 100
    pub status: CompatStatus,     // "Runs", "Partial", "Unsupported"
    pub supported_functions: Vec<String>,
    pub unsupported_functions: Vec<String>,
    pub hazardous_functions: Vec<String>,
    pub detected_toolboxes: Vec<String>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompatStatus {
    Runs,
    Partial,
    Unsupported,
}

/// Extracts MATLAB code from `matlab/document.xml`.
pub fn extract_code_from_document_xml(xml: &str) -> Result<String, LangError> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut collected = String::new();

    while let Ok(event) = reader.read_event() {
        match event {
            Event::CData(e) => {
                let text = String::from_utf8_lossy(e.as_ref()).to_string();
                if text.contains("classdef") {
                    return Ok(text);
                }
                collected.push_str(&text);
                collected.push('\n');
            }
            Event::Text(e) => {
                if let Ok(text) = e.unescape() {
                    if text.contains("classdef") {
                        collected.push_str(&text);
                        collected.push('\n');
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    if collected.contains("classdef") {
        Ok(collected)
    } else {
        // Fallback: check raw text for CDATA marker
        if let Some(start) = xml.find("<![CDATA[") {
            if let Some(end) = xml[start..].find("]]>") {
                let inner = &xml[start + 9..start + end];
                return Ok(inner.trim().to_string());
            }
        }
        Ok(xml.to_string())
    }
}

/// Parses an App Designer `classdef` into an `AppClassDef` structure.
pub fn parse_app_classdef(source: &str) -> Result<AppClassDef, LangError> {
    let clean = source.trim();
    if !clean.contains("classdef") {
        return Err(LangError::MissingClassDef);
    }

    let mut lines = clean.lines().peekable();
    let mut class_name = String::new();
    let mut super_class = None;

    // 1. Locate classdef declaration
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("classdef ") {
            let parts: Vec<&str> = rest.split('<').collect();
            class_name = parts[0].trim().to_string();
            if parts.len() > 1 {
                super_class = Some(parts[1].trim().to_string());
            }
            break;
        }
    }

    if class_name.is_empty() {
        return Err(LangError::MissingClassDef);
    }

    let mut properties = Vec::new();
    let mut methods = HashMap::new();

    // 2. Scan properties and methods blocks
    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if trimmed.starts_with("properties") {
            let access = parse_access_modifier(trimmed);
            for prop_line in lines.by_ref() {
                let p_trimmed = prop_line.trim();
                if p_trimmed == "end" {
                    break;
                }
                if p_trimmed.is_empty() || p_trimmed.starts_with('%') {
                    continue;
                }

                let tokens: Vec<&str> = p_trimmed.split_whitespace().collect();
                if !tokens.is_empty() {
                    let prop_name = tokens[0].trim_end_matches(';').to_string();
                    let type_ann = if tokens.len() > 1 && !tokens[1].starts_with('=') {
                        Some(tokens[1].trim_end_matches(';').to_string())
                    } else {
                        None
                    };

                    let init_val = p_trimmed.find('=').map(|eq_pos| {
                        p_trimmed[eq_pos + 1..]
                            .trim()
                            .trim_end_matches(';')
                            .to_string()
                    });

                    properties.push(AppProperty {
                        name: prop_name,
                        access,
                        type_annotation: type_ann,
                        initial_value: init_val,
                    });
                }
            }
        } else if trimmed.starts_with("methods") {
            let mut method_block_lines = Vec::new();
            let mut depth = 1;

            for m_line in lines.by_ref() {
                let m_trim = m_line.trim();
                if m_trim.starts_with("function") {
                    depth += 1;
                    method_block_lines.push(m_line);
                } else if m_trim == "end" {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    method_block_lines.push(m_line);
                } else {
                    if m_trim.starts_with("if ")
                        || m_trim.starts_with("for ")
                        || m_trim.starts_with("while ")
                        || m_trim.starts_with("switch ")
                        || m_trim.starts_with("try")
                    {
                        depth += 1;
                    }
                    method_block_lines.push(m_line);
                }
            }

            // Parse individual methods inside method_block_lines
            parse_methods_block(&method_block_lines.join("\n"), &mut methods)?;
        }
    }

    Ok(AppClassDef {
        name: class_name,
        super_class,
        properties,
        methods,
        raw_source: source.to_string(),
    })
}

fn parse_access_modifier(line: &str) -> AccessModifier {
    let lower = line.to_lowercase();
    if lower.contains("private") {
        AccessModifier::Private
    } else if lower.contains("protected") {
        AccessModifier::Protected
    } else {
        AccessModifier::Public
    }
}

fn parse_methods_block(
    block_code: &str,
    methods: &mut HashMap<String, AppMethod>,
) -> Result<(), LangError> {
    // Parse functions with rustlab_parse
    let stmts = rustlab_parse::parse(block_code).map_err(|e| LangError::Parse(e.to_string()))?;

    for stmt in stmts {
        if let Stmt::Function(f) = stmt {
            let role = if f.name.eq_ignore_ascii_case("startupfcn") {
                MethodRole::StartupFcn
            } else if f.name.ends_with("Pushed")
                || f.name.ends_with("ValueChanged")
                || f.name.ends_with("Clicked")
                || f.name.ends_with("Callback")
            {
                MethodRole::Callback { event_type: None }
            } else {
                MethodRole::Helper
            };

            methods.insert(
                f.name.clone(),
                AppMethod {
                    name: f.name,
                    inputs: f.inputs,
                    outputs: f.outputs,
                    role,
                    body: f.body,
                    span: f.span,
                },
            );
        }
    }

    Ok(())
}

/// Statically analyzes an `AppClassDef` for compatibility and security risks.
pub fn analyze_compatibility(app: &AppClassDef) -> CompatReport {
    let mut supported = HashSet::new();
    let mut unsupported = HashSet::new();
    let mut hazardous = HashSet::new();
    let toolboxes = HashSet::new();
    let mut diagnostics = Vec::new();

    let known_supported = [
        "zeros",
        "ones",
        "eye",
        "rand",
        "randn",
        "size",
        "length",
        "numel",
        "ndims",
        "sum",
        "prod",
        "cumsum",
        "mean",
        "std",
        "min",
        "max",
        "sin",
        "cos",
        "tan",
        "asin",
        "acos",
        "atan",
        "exp",
        "log",
        "log10",
        "sqrt",
        "abs",
        "linspace",
        "reshape",
        "repmat",
        "sort",
        "find",
        "unique",
        "any",
        "all",
        "strcat",
        "sprintf",
        "fprintf",
        "num2str",
        "str2double",
        "disp",
        "plot",
        "scatter",
        "bar",
        "title",
        "xlabel",
        "ylabel",
        "grid",
        "hold",
        "xlim",
        "ylim",
        "uialert",
        "uiconfirm",
        "tic",
        "toc",
        "pause",
        "drawnow",
        "inv",
        "det",
    ];

    let known_hazardous = [
        "system",
        "dos",
        "unix",
        "delete",
        "rmdir",
        "web",
        "urlread",
        "actxserver",
        "loadlibrary",
        "calllib",
    ];

    let known_dynamic = [
        "eval", "evalc", "evalin", "assignin", "feval", "str2func", "run", "source", "builtin",
    ];

    let mut unanalyzable = HashSet::new();

    for (m_name, method) in &app.methods {
        for stmt in &method.body {
            collect_call_names_stmt(stmt, &mut |name| {
                if known_hazardous.contains(&name) {
                    hazardous.insert(name.to_string());
                    diagnostics.push(format!(
                        "Method '{}' calls hazardous/restricted function '{}'",
                        m_name, name
                    ));
                } else if known_dynamic.contains(&name) {
                    unanalyzable.insert(name.to_string());
                    hazardous.insert(name.to_string());
                    diagnostics.push(format!(
                        "Method '{}' contains unanalyzable dynamic call '{}'",
                        m_name, name
                    ));
                } else if known_supported.contains(&name) {
                    supported.insert(name.to_string());
                } else if name.starts_with("app.") || name == "app" {
                    // Property/Component handle access
                } else {
                    unsupported.insert(name.to_string());
                    diagnostics.push(format!(
                        "Method '{}' calls unsupported function '{}'",
                        m_name, name
                    ));
                }
            });
        }
    }

    // Scoring Formula:
    // Base: 100 points
    // Deductions: -25 per hazardous call, -15 per unanalyzable dynamic call, -5 per unsupported builtin
    let penalty = (hazardous.len() * 25) + (unanalyzable.len() * 15) + (unsupported.len() * 5);
    let score = 100u32.saturating_sub(penalty as u32);

    let status = if !hazardous.is_empty() || score < 50 {
        CompatStatus::Unsupported
    } else if score < 90 || !unanalyzable.is_empty() {
        CompatStatus::Partial
    } else {
        CompatStatus::Runs
    };

    let mut supported_vec: Vec<String> = supported.into_iter().collect();
    supported_vec.sort();
    let mut unsupported_vec: Vec<String> = unsupported.into_iter().collect();
    unsupported_vec.sort();
    let mut hazardous_vec: Vec<String> = hazardous.into_iter().collect();
    hazardous_vec.sort();
    let mut toolboxes_vec: Vec<String> = toolboxes.into_iter().collect();
    toolboxes_vec.sort();
    diagnostics.sort();

    CompatReport {
        compatibility_score: score,
        status,
        supported_functions: supported_vec,
        unsupported_functions: unsupported_vec,
        hazardous_functions: hazardous_vec,
        detected_toolboxes: toolboxes_vec,
        diagnostics,
    }
}

fn collect_call_names_stmt<F: FnMut(&str)>(stmt: &Stmt, cb: &mut F) {
    match stmt {
        Stmt::Expr { expr, .. } => collect_call_names_expr(expr, cb),
        Stmt::Assignment { value, .. } => collect_call_names_expr(value, cb),
        Stmt::If {
            branches,
            else_branch,
            ..
        } => {
            for (cond, body) in branches {
                collect_call_names_expr(cond, cb);
                for s in body {
                    collect_call_names_stmt(s, cb);
                }
            }
            if let Some(eb) = else_branch {
                for s in eb {
                    collect_call_names_stmt(s, cb);
                }
            }
        }
        Stmt::For { expr, body, .. } => {
            collect_call_names_expr(expr, cb);
            for s in body {
                collect_call_names_stmt(s, cb);
            }
        }
        Stmt::While { cond, body, .. } => {
            collect_call_names_expr(cond, cb);
            for s in body {
                collect_call_names_stmt(s, cb);
            }
        }
        _ => {}
    }
}

fn collect_call_names_expr<F: FnMut(&str)>(expr: &rustlab_parse::ast::Expr, cb: &mut F) {
    use rustlab_parse::ast::Expr;
    match expr {
        Expr::Index { callee, args, .. } => {
            if let Expr::Ident(name, _) = &**callee {
                cb(name);
            } else {
                collect_call_names_expr(callee, cb);
            }
            for a in args {
                collect_call_names_expr(a, cb);
            }
        }
        Expr::Binary { left, right, .. } => {
            collect_call_names_expr(left, cb);
            collect_call_names_expr(right, cb);
        }
        Expr::Unary { expr, .. } => {
            collect_call_names_expr(expr, cb);
        }
        _ => {}
    }
}
