use std::{fs::File, io::Read, path::Path};

use zip::ZipArchive;

use crate::{
    error::{Error, Result},
    model::SaveSummary,
};

const MAX_META_BYTES: u64 = 1_048_576;
const MAX_GAMESTATE_BYTES: u64 = 128 * 1_048_576;

#[derive(Debug, Clone)]
pub(crate) enum ClausewitzValue {
    Atom(String),
    Block(Vec<ClausewitzNode>),
}

#[derive(Debug, Clone)]
pub(crate) enum ClausewitzNode {
    Pair(String, ClausewitzValue),
    Value(ClausewitzValue),
}

pub fn parse_save(path: impl AsRef<Path>) -> Result<SaveSummary> {
    let path = path.as_ref();
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    let meta = read_zip_entry_text(&mut archive, "meta", MAX_META_BYTES)?;
    let gamestate = read_zip_entry_text(&mut archive, "gamestate", MAX_GAMESTATE_BYTES)?;

    let meta_root = parse_clausewitz(&meta)?;
    let game_root = parse_clausewitz(&gamestate)?;

    let mut summary = SaveSummary {
        path: path.to_path_buf(),
        ..Default::default()
    };

    summary.version =
        first_atom(&meta_root, "version").or_else(|| first_atom(&game_root, "version"));
    summary.date = first_atom(&meta_root, "date").or_else(|| first_atom(&game_root, "date"));
    summary.name = first_atom(&meta_root, "name").or_else(|| first_atom(&game_root, "name"));
    summary.required_dlcs = find_value_by_key(&meta_root, "required_dlcs")
        .map(collect_atoms)
        .unwrap_or_default();
    summary.ironman =
        first_bool(&meta_root, "ironman").or_else(|| first_bool(&game_root, "ironman"));
    summary.cheated_on_save = first_bool(&game_root, "cheated_on_save");

    summary.player_country = find_value_by_key(&game_root, "player")
        .and_then(|value| find_value_by_key(value, "country"))
        .and_then(atom_string);

    if let Some(country_id) = summary.player_country.as_deref() {
        if let Some(country_root) = top_level_value_by_key(&game_root, "country") {
            if let Some(country_value) = direct_entry(country_root, country_id) {
                let government = direct_value_by_key(country_value, "government");
                summary.authority = government
                    .and_then(|value| direct_value_by_key(value, "authority"))
                    .or_else(|| direct_value_by_key(country_value, "authority"))
                    .and_then(atom_string);
                summary.government_type = government
                    .and_then(|value| direct_value_by_key(value, "type"))
                    .or_else(|| direct_value_by_key(country_value, "government_type"))
                    .and_then(atom_string);
                summary.origin = government
                    .and_then(|value| direct_value_by_key(value, "origin"))
                    .or_else(|| direct_value_by_key(country_value, "origin"))
                    .and_then(atom_string);
                summary.ethics = direct_value_by_key(country_value, "ethos")
                    .or_else(|| direct_value_by_key(country_value, "ethics"))
                    .map(collect_atoms)
                    .unwrap_or_default();
                summary.civics = government
                    .and_then(|value| direct_value_by_key(value, "civics"))
                    .or_else(|| direct_value_by_key(country_value, "civics"))
                    .map(collect_atoms)
                    .unwrap_or_default();
                summary.founder_species_ref =
                    direct_value_by_key(country_value, "founder_species_ref")
                        .or_else(|| direct_value_by_key(country_value, "founder_species"))
                        .or_else(|| direct_value_by_key(country_value, "species"))
                        .and_then(atom_string);

                // Extract discovery, progression, and action facts
                summary.discovery = Some(crate::extract_discovery::extract_discovery_facts(
                    &game_root,
                    country_value,
                ));
                summary.progression = Some(crate::extract_progression::extract_progression_facts(
                    &game_root,
                    country_value,
                ));
                summary.actions = Some(crate::extract_action::extract_action_facts(
                    &game_root,
                    country_value,
                    country_id,
                ));
            }
        }
    }

    if let Some(species_ref) = summary.founder_species_ref.as_deref() {
        if let Some(species_db) = top_level_value_by_key(&game_root, "species_db") {
            if let Some(species_value) = direct_entry(species_db, species_ref) {
                summary.founder_species_class = direct_value_by_key(species_value, "class")
                    .or_else(|| direct_value_by_key(species_value, "species_class"))
                    .and_then(atom_string);
                summary.founder_species_portrait =
                    direct_value_by_key(species_value, "portrait").and_then(atom_string);
                summary.founder_species_traits = direct_value_by_key(species_value, "traits")
                    .map(collect_atoms)
                    .unwrap_or_default();
            }
        }
    }

    Ok(summary)
}

fn read_zip_entry_text(
    archive: &mut ZipArchive<File>,
    name: &str,
    max_bytes: u64,
) -> Result<String> {
    let mut file = archive.by_name(name)?;
    if file.size() > max_bytes {
        return Err(Error::Parse(format!(
            "zip entry {name} is {} bytes, exceeds {max_bytes} byte safety limit",
            file.size()
        )));
    }
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(String::from_utf8(buf)?)
}

fn parse_clausewitz(text: &str) -> Result<ClausewitzValue> {
    let tokens = tokenize(text);
    let (nodes, consumed) = parse_nodes(&tokens, 0, false)?;
    if consumed != tokens.len() {
        return Err(Error::Parse(
            "unexpected trailing Clausewitz tokens".to_string(),
        ));
    }
    Ok(ClausewitzValue::Block(nodes))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    LBrace,
    RBrace,
    Equals,
    Atom(String),
}

fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            c if c.is_whitespace() => continue,
            '#' => {
                for next in chars.by_ref() {
                    if next == '\n' {
                        break;
                    }
                }
            }
            '{' => tokens.push(Token::LBrace),
            '}' => tokens.push(Token::RBrace),
            '=' => tokens.push(Token::Equals),
            '"' => tokens.push(Token::Atom(parse_quoted(&mut chars))),
            _ => {
                let mut value = String::new();
                value.push(ch);
                while let Some(next) = chars.peek().copied() {
                    if next.is_whitespace() || matches!(next, '{' | '}' | '=' | '#') {
                        break;
                    }
                    value.push(next);
                    chars.next();
                }
                tokens.push(Token::Atom(value));
            }
        }
    }

    tokens
}

fn parse_quoted(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut value = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => break,
            '\\' => {
                if let Some(escaped) = chars.next() {
                    value.push(escaped);
                }
            }
            other => value.push(other),
        }
    }
    value
}

fn parse_nodes(
    tokens: &[Token],
    mut pos: usize,
    stop_on_rbrace: bool,
) -> Result<(Vec<ClausewitzNode>, usize)> {
    let mut nodes = Vec::new();

    while pos < tokens.len() {
        match tokens.get(pos) {
            Some(Token::RBrace) if stop_on_rbrace => return Ok((nodes, pos + 1)),
            Some(Token::RBrace) => {
                return Err(Error::Parse("unexpected closing brace".to_string()))
            }
            Some(Token::LBrace) => {
                let (value, next) = parse_value(tokens, pos)?;
                nodes.push(ClausewitzNode::Value(value));
                pos = next;
            }
            Some(Token::Atom(key)) if matches!(tokens.get(pos + 1), Some(Token::Equals)) => {
                let (value, next) = parse_value(tokens, pos + 2)?;
                nodes.push(ClausewitzNode::Pair(key.clone(), value));
                pos = next;
            }
            Some(_) => {
                let (value, next) = parse_value(tokens, pos)?;
                nodes.push(ClausewitzNode::Value(value));
                pos = next;
            }
            None => break,
        }
    }

    if stop_on_rbrace {
        return Err(Error::Parse("missing closing brace".to_string()));
    }

    Ok((nodes, pos))
}

fn parse_value(tokens: &[Token], pos: usize) -> Result<(ClausewitzValue, usize)> {
    match tokens.get(pos) {
        Some(Token::Atom(value)) => Ok((ClausewitzValue::Atom(value.clone()), pos + 1)),
        Some(Token::LBrace) => {
            let (nodes, next) = parse_nodes(tokens, pos + 1, true)?;
            Ok((ClausewitzValue::Block(nodes), next))
        }
        Some(Token::RBrace) => Err(Error::Parse("unexpected closing brace".to_string())),
        Some(Token::Equals) => Err(Error::Parse("unexpected equals sign".to_string())),
        None => Err(Error::Parse(
            "unexpected end of Clausewitz input".to_string(),
        )),
    }
}

fn find_value_by_key<'a>(value: &'a ClausewitzValue, key: &str) -> Option<&'a ClausewitzValue> {
    match value {
        ClausewitzValue::Atom(_) => None,
        ClausewitzValue::Block(nodes) => {
            for node in nodes {
                if let ClausewitzNode::Pair(found_key, found_value) = node {
                    if found_key == key {
                        return Some(found_value);
                    }
                }
            }

            for node in nodes {
                match node {
                    ClausewitzNode::Pair(_, child) | ClausewitzNode::Value(child) => {
                        if let Some(found) = find_value_by_key(child, key) {
                            return Some(found);
                        }
                    }
                }
            }

            None
        }
    }
}

fn direct_entry<'a>(value: &'a ClausewitzValue, key: &str) -> Option<&'a ClausewitzValue> {
    match value {
        ClausewitzValue::Block(nodes) => nodes.iter().find_map(|node| match node {
            ClausewitzNode::Pair(found_key, found_value) if found_key == key => Some(found_value),
            _ => None,
        }),
        ClausewitzValue::Atom(_) => None,
    }
}

fn top_level_value_by_key<'a>(
    value: &'a ClausewitzValue,
    key: &str,
) -> Option<&'a ClausewitzValue> {
    direct_value_by_key(value, key)
}

fn direct_value_by_key<'a>(value: &'a ClausewitzValue, key: &str) -> Option<&'a ClausewitzValue> {
    match value {
        ClausewitzValue::Block(nodes) => nodes.iter().find_map(|node| match node {
            ClausewitzNode::Pair(found_key, found_value) if found_key == key => Some(found_value),
            _ => None,
        }),
        ClausewitzValue::Atom(_) => None,
    }
}

fn first_atom(value: &ClausewitzValue, key: &str) -> Option<String> {
    find_value_by_key(value, key).and_then(atom_string)
}

fn first_bool(value: &ClausewitzValue, key: &str) -> Option<bool> {
    find_value_by_key(value, key).and_then(bool_from_value)
}

fn atom_string(value: &ClausewitzValue) -> Option<String> {
    match value {
        ClausewitzValue::Atom(atom) if !atom.is_empty() => Some(atom.clone()),
        _ => None,
    }
}

fn bool_from_value(value: &ClausewitzValue) -> Option<bool> {
    match value {
        ClausewitzValue::Atom(atom) => match atom.as_str() {
            "yes" | "true" | "1" => Some(true),
            "no" | "false" | "0" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn collect_atoms(value: &ClausewitzValue) -> Vec<String> {
    let mut atoms = Vec::new();
    collect_atoms_into(value, &mut atoms);
    dedupe_preserve_order(atoms)
}

fn collect_atoms_into(value: &ClausewitzValue, atoms: &mut Vec<String>) {
    match value {
        ClausewitzValue::Atom(atom) if !atom.is_empty() => atoms.push(atom.clone()),
        ClausewitzValue::Atom(_) => {}
        ClausewitzValue::Block(nodes) => {
            for node in nodes {
                match node {
                    ClausewitzNode::Pair(_, child) | ClausewitzNode::Value(child) => {
                        collect_atoms_into(child, atoms)
                    }
                }
            }
        }
    }
}

fn dedupe_preserve_order(values: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for value in values {
        if !out.iter().any(|existing| existing == &value) {
            out.push(value);
        }
    }
    out
}

// ── Query helpers ──────────────────────────────────────────

/// Navigate a path through the Clausewitz AST using direct child lookups.
/// Returns None if any segment is missing.
#[allow(dead_code)]
pub(crate) fn query_path<'a>(
    value: &'a ClausewitzValue,
    path: &[&str],
) -> Option<&'a ClausewitzValue> {
    let mut current = value;
    for segment in path {
        current = direct_entry(current, segment)?;
    }
    Some(current)
}

/// Count the number of direct entries (Pair or Value nodes) in a block.
/// Returns 0 if the value is not a Block.
#[allow(dead_code)]
pub(crate) fn count_entries(value: &ClausewitzValue) -> usize {
    match value {
        ClausewitzValue::Block(nodes) => nodes.len(),
        ClausewitzValue::Atom(_) => 0,
    }
}

/// Parse a numeric value from a Clausewitz atom string.
/// Handles integers and floats. Returns None if not a valid number.
#[allow(dead_code)]
pub(crate) fn parse_f64(value: &ClausewitzValue) -> Option<f64> {
    match value {
        ClausewitzValue::Atom(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

/// Navigate a path and parse the final value as f64.
#[allow(dead_code)]
pub(crate) fn query_f64(value: &ClausewitzValue, path: &[&str]) -> Option<f64> {
    query_path(value, path).and_then(parse_f64)
}

/// Navigate a path and extract the final value as a string atom.
#[allow(dead_code)]
pub(crate) fn query_atom(value: &ClausewitzValue, path: &[&str]) -> Option<String> {
    query_path(value, path).and_then(atom_string)
}

/// Navigate a path and extract the final value as a bool.
#[allow(dead_code)]
pub(crate) fn query_bool(value: &ClausewitzValue, path: &[&str]) -> Option<bool> {
    query_path(value, path).and_then(bool_from_value)
}

/// Count entries at a given path.
#[allow(dead_code)]
pub(crate) fn query_count(value: &ClausewitzValue, path: &[&str]) -> Option<usize> {
    query_path(value, path).map(count_entries)
}

/// Collect all atoms at a given path.
#[allow(dead_code)]
pub(crate) fn query_atoms(value: &ClausewitzValue, path: &[&str]) -> Vec<String> {
    query_path(value, path)
        .map(collect_atoms)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_path_simple() {
        let ast = parse_clausewitz("a = { b = { c = hello } }").unwrap();
        assert_eq!(
            query_atom(&ast, &["a", "b", "c"]),
            Some("hello".to_string())
        );
    }

    #[test]
    fn test_query_path_missing() {
        let ast = parse_clausewitz("a = { b = hello }").unwrap();
        assert!(query_path(&ast, &["a", "x"]).is_none());
    }

    #[test]
    fn test_count_entries() {
        let ast = parse_clausewitz("items = { a b c }").unwrap();
        let items = direct_entry(&ast, "items").unwrap();
        assert_eq!(count_entries(items), 3);
    }

    #[test]
    fn test_parse_f64() {
        let ast = parse_clausewitz("value = 42.5").unwrap();
        let v = direct_entry(&ast, "value").unwrap();
        assert_eq!(parse_f64(v), Some(42.5));
    }

    #[test]
    fn test_parse_f64_int() {
        let ast = parse_clausewitz("value = 42").unwrap();
        let v = direct_entry(&ast, "value").unwrap();
        assert_eq!(parse_f64(v), Some(42.0));
    }

    #[test]
    fn test_parse_f64_not_a_number() {
        let ast = parse_clausewitz("value = hello").unwrap();
        let v = direct_entry(&ast, "value").unwrap();
        assert_eq!(parse_f64(v), None);
    }

    #[test]
    fn test_query_count() {
        let ast = parse_clausewitz("planets = { 0 = {} 1 = {} 2 = {} }").unwrap();
        assert_eq!(query_count(&ast, &["planets"]), Some(3));
    }

    #[test]
    fn test_query_bool() {
        let ast = parse_clausewitz("flag = yes").unwrap();
        assert_eq!(query_bool(&ast, &["flag"]), Some(true));
    }

    #[test]
    fn test_query_atoms() {
        let ast = parse_clausewitz("list = { a b c }").unwrap();
        assert_eq!(query_atoms(&ast, &["list"]), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_query_f64() {
        let ast = parse_clausewitz("a = { b = 123.45 }").unwrap();
        assert_eq!(query_f64(&ast, &["a", "b"]), Some(123.45));
    }

    #[test]
    fn test_count_entries_on_atom() {
        let ast = parse_clausewitz("value = hello").unwrap();
        let v = direct_entry(&ast, "value").unwrap();
        assert_eq!(count_entries(v), 0);
    }
}
