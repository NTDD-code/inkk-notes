//! Core headless Flowchart Engine.
//! Strictly zero GUI dependencies. Can run in tests, servers, CLI and web workers.

pub mod ast;
pub mod check;
pub mod export;
pub mod ir;
pub mod layout;
pub mod lexer;
pub mod parser;
pub mod print;
pub mod route;

#[cfg(test)]
mod tests;
