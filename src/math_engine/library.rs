//! Library and Package Management for Inkk MathEngine
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::math_engine::ast::{FunctionMetadata, StmtKind, UserFunctionAst};
use crate::math_engine::error::MathError;
use crate::math_engine::eval::Evaluator;
use crate::math_engine::parser::Parser;
use crate::math_engine::registry::FunctionKind;
use crate::math_engine::token::Lexer;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryManifest {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub requires_inkk: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileLoadReport {
    pub file: String,
    pub success: bool,
    pub error: Option<String>,
    pub function_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LibraryLoadReport {
    pub library_name: String,
    pub files: Vec<FileLoadReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedFunction {
    pub name: String,
    pub source: String,
    pub library: Option<String>,
    pub version: Option<String>,
    pub hash: String,
}

pub struct LibraryManager {
    base_dir: PathBuf,
}

impl LibraryManager {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    /// Load a single .m source code as a public function and its local functions
    pub fn parse_function_source(
        filename: &str,
        source: &str,
        library_name: Option<&str>,
        evaluator: &Evaluator,
    ) -> Result<(UserFunctionAst, FunctionMetadata), (usize, String)> {
        let expected_name = match filename.strip_suffix(".m") {
            Some(stem) => stem,
            None => filename,
        };

        // Check if name is a built-in function
        if let Some(entry) = evaluator.registry.get(expected_name) {
            if matches!(entry.kind, FunctionKind::Builtin(_)) {
                return Err((
                    1,
                    format!(
                        "Cannot define '{}': name is a built-in function.",
                        expected_name
                    ),
                ));
            }
        }

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|e| (1, e.raw_message()))?;
        let comments = lexer.comments;
        let mut parser = Parser::new_with_comments(tokens, comments);

        let script = parser.parse_script().map_err(|e| {
            let line = match e {
                MathError::ParseError { line, .. } => line,
                MathError::FeatureNotSupported { line, .. } => line,
                _ => 1,
            };
            (line, e.raw_message())
        })?;

        let mut stmts = script.stmts.into_iter();
        let first_stmt = stmts
            .next()
            .ok_or((1, "File contains no statements".to_string()))?;

        let (mut main_ast, main_meta) = match first_stmt.kind {
            StmtKind::FunctionDef { ast, metadata } => (ast, metadata),
            _ => {
                return Err((
                    first_stmt.line,
                    format!(
                        "File '{}' must define function '{}'",
                        filename, expected_name
                    ),
                ))
            }
        };

        if main_ast.name != expected_name {
            return Err((
                first_stmt.line,
                format!(
                    "Function name '{}' does not match filename '{}'",
                    main_ast.name, filename
                ),
            ));
        }

        // Subsequent functions in the file become local functions
        for stmt in stmts {
            if let StmtKind::FunctionDef { ast: local_ast, .. } = stmt.kind {
                main_ast
                    .local_functions
                    .insert(local_ast.name.clone(), local_ast);
            }
        }

        main_ast.library = library_name.map(|s| s.to_string());

        Ok((main_ast, main_meta))
    }

    /// Load all libraries from `base_dir` into the given evaluator
    pub fn load_all(&self, evaluator: &mut Evaluator) -> Vec<LibraryLoadReport> {
        let mut reports = Vec::new();
        if !self.base_dir.exists() {
            return reports;
        }

        // 1. _local/ directory
        let local_dir = self.base_dir.join("_local");
        if local_dir.is_dir() {
            let report = self.load_directory(&local_dir, None, evaluator);
            reports.push(report);
        }

        // 2. <library_name>/ directories
        if let Ok(entries) = fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if dir_name == "_local" {
                        continue;
                    }
                    let report = self.load_directory(&path, Some(dir_name), evaluator);
                    reports.push(report);
                }
            }
        }

        reports
    }

    /// Load a directory of .m files
    pub fn load_directory(
        &self,
        dir: &Path,
        lib_name: Option<&str>,
        evaluator: &mut Evaluator,
    ) -> LibraryLoadReport {
        let name_str = lib_name.unwrap_or("_local").to_string();
        let mut file_reports = Vec::new();

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("m") {
                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    let content = match fs::read_to_string(&path) {
                        Ok(c) => c,
                        Err(e) => {
                            file_reports.push(FileLoadReport {
                                file: file_name,
                                success: false,
                                error: Some(format!("Failed to read file: {}", e)),
                                function_name: None,
                            });
                            continue;
                        }
                    };

                    match Self::parse_function_source(&file_name, &content, lib_name, evaluator) {
                        Ok((ast, meta)) => {
                            let fn_name = ast.name.clone();
                            match evaluator.registry.register_user(ast, meta) {
                                Ok(_) => {
                                    file_reports.push(FileLoadReport {
                                        file: file_name,
                                        success: true,
                                        error: None,
                                        function_name: Some(fn_name),
                                    });
                                }
                                Err(err) => {
                                    file_reports.push(FileLoadReport {
                                        file: file_name,
                                        success: false,
                                        error: Some(err.raw_message()),
                                        function_name: Some(fn_name),
                                    });
                                }
                            }
                        }
                        Err((line, err_msg)) => {
                            file_reports.push(FileLoadReport {
                                file: file_name.clone(),
                                success: false,
                                error: Some(format!("{}:{}: {}", file_name, line, err_msg)),
                                function_name: None,
                            });
                        }
                    }
                }
            }
        }

        LibraryLoadReport {
            library_name: name_str,
            files: file_reports,
        }
    }
}

/// Helper to hash function source code
pub fn compute_source_hash(source: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Load embedded functions into a local evaluator scope (for shared whiteboard)
pub fn load_embedded_functions(
    evaluator: &mut Evaluator,
    funcs: &[EmbeddedFunction],
) -> Vec<Result<String, String>> {
    let mut results = Vec::new();
    for ef in funcs {
        match LibraryManager::parse_function_source(
            &format!("{}.m", ef.name),
            &ef.source,
            ef.library.as_deref(),
            evaluator,
        ) {
            Ok((ast, meta)) => match evaluator.registry.register_user(ast, meta) {
                Ok(_) => results.push(Ok(ef.name.clone())),
                Err(e) => results.push(Err(format!(
                    "Error loading embedded function '{}': {}",
                    ef.name, e
                ))),
            },
            Err((line, err)) => {
                results.push(Err(format!(
                    "Error parsing embedded function '{}' at line {}: {}",
                    ef.name, line, err
                )));
            }
        }
    }
    results
}
