use super::Result;
use roxmltree::{Document, Node};
use std::ops::Range;

pub fn parse(text: &str) -> Result<Document<'_>> {
    Ok(Document::parse_with_options(text, roxmltree::ParsingOptions { allow_dtd: false, nodes_limit: 8_000_000 })?)
}

pub fn child<'a, 'b>(node: Node<'a, 'b>, tag: &str) -> Option<Node<'a, 'b>> {
    node.children().find(|child| child.has_tag_name(tag))
}

pub fn value<'a, 'b>(node: Node<'a, 'b>, tag: &str) -> Option<&'a str> {
    child(node, tag).and_then(|child| child.attribute("v"))
}

pub fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
        .replace('\n', "&#10;")
        .replace('\r', "&#13;")
}

pub fn leaf(tag: &str, value: &str) -> String {
    format!("<{tag} v=\"{}\"/>", escape(value))
}

pub fn title(identifier: &str) -> String {
    identifier.replace(['_', '-'], " ")
}

pub fn identity(path: &str) -> String {
    let hash = |seed: u64| path.bytes().fold(seed, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3));
    let bits = (u128::from(hash(0xcbf29ce484222325)) << 64) | u128::from(hash(0x9e3779b97f4a7c15));
    let id = format!("{:032x}", (bits & !(0xf000_u128 << 64) & !(0xc000_u128 << 48)) | (0x4000_u128 << 64) | (0x8000_u128 << 48));
    format!("{{{}-{}-{}-{}-{}}}", &id[..8], &id[8..12], &id[12..16], &id[16..20], &id[20..])
}

pub struct Edit {
    pub range: Range<usize>,
    pub replacement: String,
}

pub fn set(edits: &mut Vec<Edit>, node: Node<'_, '_>, value: &str) {
    edits.push(Edit { range: node.range(), replacement: leaf(node.tag_name().name(), value) });
}

pub fn apply(text: &str, mut edits: Vec<Edit>) -> Result<String> {
    edits.sort_by_key(|edit| (edit.range.start, edit.range.end));
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;
    for edit in edits {
        if edit.range.start < cursor || edit.range.end < edit.range.start {
            return Err("overlapping XML edits".into());
        }
        output.push_str(text.get(cursor..edit.range.start).ok_or("invalid XML edit boundary")?);
        output.push_str(&edit.replacement);
        cursor = edit.range.end;
    }
    output.push_str(text.get(cursor..).ok_or("invalid XML edit boundary")?);
    Ok(output)
}

pub fn relative(owner: &str, target: &str) -> Result<String> {
    for path in [owner, target] {
        materialcraft_formats::sbs::library::relative_path(&format!("sbs://{path}"))?;
    }
    let mut from: Vec<_> = owner.split('/').collect();
    from.pop();
    let to: Vec<_> = target.split('/').collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    Ok(std::iter::repeat_n("..", from.len() - common).chain(to.into_iter().skip(common)).collect::<Vec<_>>().join("/"))
}
