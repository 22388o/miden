use crate::{v1::block_parser::parse_block_body, AssemblyError};
use std::collections::BTreeMap;
use vm_core::v1::program::{blocks::CodeBlock, Script};

mod block_parser;

mod op_parser;
use op_parser::parse_op_token;

mod token_stream;
use token_stream::TokenStream;

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================

const BEGIN: &str = "begin";
const PROC: &str = "proc";
const END: &str = "end";

// ASSEMBLY COMPILER
// ================================================================================================

/// TODO: add comments
pub fn compile_script(source: &str) -> Result<Script, AssemblyError> {
    let mut tokens = TokenStream::new(source);
    let mut proc_map = BTreeMap::new();

    // parse procedures and add them to the procedure map; procedures are parsed in the order
    // in which they appear in the source, and thus, procedures which come later may invoke
    // preceding procedures
    while let Some(token) = tokens.read() {
        match token[0] {
            PROC => parse_proc(&mut tokens, &mut proc_map)?,
            _ => break,
        }
    }

    // make sure script body is present
    let next_token = tokens
        .read()
        .ok_or_else(|| AssemblyError::missing_begin(tokens.pos()))?;
    if next_token[0] != BEGIN {
        return Err(AssemblyError::dangling_ops_after_proc(
            next_token,
            tokens.pos(),
        ));
    }

    // parse script body and return the resulting script
    let script_root = parse_script(&mut tokens, &proc_map)?;
    Ok(Script::new(script_root))
}

// PARSERS
// ================================================================================================

/// TODO: add comments
fn parse_script(
    tokens: &mut TokenStream,
    proc_map: &BTreeMap<String, CodeBlock>,
) -> Result<CodeBlock, AssemblyError> {
    let script_start = tokens.pos();
    // consume the 'begin' token
    let header = tokens.read().expect("missing script header");
    validate_begin_token(header, tokens.pos())?;
    tokens.advance();

    // parse the script body
    let root = block_parser::parse_block_body(tokens, proc_map)?;

    // consume the 'end' token
    match tokens.read() {
        None => Err(AssemblyError::unmatched_begin(script_start)),
        Some(token) => match token[0] {
            END => validate_end_token(token, tokens.pos()),
            _ => Err(AssemblyError::unmatched_begin(script_start)),
        },
    }?;
    tokens.advance();

    // make sure there are no instructions after the end
    if let Some(token) = tokens.read() {
        return Err(AssemblyError::dangling_ops_after_script(
            token,
            tokens.pos(),
        ));
    }

    Ok(root)
}

/// TODO: add comments
fn parse_proc(
    tokens: &mut TokenStream,
    proc_map: &mut BTreeMap<String, CodeBlock>,
) -> Result<(), AssemblyError> {
    let proc_start = tokens.pos();

    // read procedure name and consume the procedure header token
    let header = tokens.read().expect("missing procedure header");
    let label = validate_proc_token(header, tokens.pos())?;
    if proc_map.contains_key(&label) {
        return Err(AssemblyError::duplicate_proc_label(tokens.pos(), &label));
    }
    tokens.advance();

    // parse procedure body
    let root = parse_block_body(tokens, proc_map)?;

    // consume the 'end' token
    match tokens.read() {
        None => Err(AssemblyError::unmatched_proc(proc_start, &label)),
        Some(token) => match token[0] {
            END => validate_end_token(token, tokens.pos()),
            _ => Err(AssemblyError::unmatched_proc(proc_start, &label)),
        },
    }?;
    tokens.advance();

    // add the procedure to the procedure map and return
    proc_map.insert(label, root);
    Ok(())
}

// HELPER FUNCTIONS
// ================================================================================================

fn validate_begin_token(token: &[&str], pos: usize) -> Result<(), AssemblyError> {
    assert_eq!(BEGIN, token[0], "not a begin");
    if token.len() > 1 {
        Err(AssemblyError::extra_param(token, pos))
    } else {
        Ok(())
    }
}

fn validate_proc_token(token: &[&str], pos: usize) -> Result<String, AssemblyError> {
    assert_eq!(PROC, token[0], "invalid procedure header");
    match token.len() {
        1 => Err(AssemblyError::missing_param(token, pos)),
        2 => validate_proc_label(token[1]),
        _ => Err(AssemblyError::extra_param(token, pos)),
    }
}

fn validate_proc_label(label: &str) -> Result<String, AssemblyError> {
    // TODO: validate name
    Ok(label.to_string())
}

fn validate_end_token(token: &[&str], pos: usize) -> Result<(), AssemblyError> {
    assert_eq!(END, token[0], "not an end");
    if token.len() > 1 {
        Err(AssemblyError::invalid_param_reason(
            token,
            pos,
            "expected end".to_string(),
        ))
    } else {
        Ok(())
    }
}
