use super::{
    Result,
    xml::{self, Edit},
};
use roxmltree::Node;
use std::collections::{BTreeMap, BTreeSet};

pub fn reposition(list: Node<'_, '_>, edits: &mut Vec<Edit>) -> Result<usize> {
    let nodes: Vec<_> = list.children().filter(Node::is_element).collect();
    if nodes.len() > 100_000 {
        return Err("canvas exceeds the node limit".into());
    }
    let mut ids = BTreeMap::new();
    for node in &nodes {
        let id = xml::value(*node, "uid").ok_or("canvas node has no uid")?;
        if ids.insert(id, *node).is_some() {
            return Err("duplicate canvas node uid".into());
        }
    }
    let mut pending = BTreeMap::new();
    let mut successors = BTreeMap::<&str, Vec<&str>>::new();
    let mut columns = BTreeMap::<&str, usize>::new();
    let mut ready = BTreeSet::new();
    for (id, node) in &ids {
        let sources: BTreeSet<_> = xml::child(*node, "connections")
            .into_iter()
            .flat_map(|c| c.children())
            .filter(Node::is_element)
            .filter_map(|connection| xml::value(connection, "connRef"))
            .collect();
        if sources.iter().any(|source| !ids.contains_key(source)) {
            return Err("canvas connection references an absent node".into());
        }
        pending.insert(*id, sources.len());
        if sources.is_empty() {
            ready.insert(*id);
        }
        for source in sources {
            successors.entry(source).or_default().push(id);
        }
    }
    let mut visited = 0;
    while let Some(id) = ready.pop_first() {
        visited += 1;
        let next = columns.get(id).copied().unwrap_or(0) + 1;
        for successor in successors.get(id).into_iter().flatten() {
            let column = columns.entry(successor).or_default();
            *column = (*column).max(next);
            let count = pending.get_mut(successor).ok_or("missing layout successor")?;
            *count = count.checked_sub(1).ok_or("layout dependency underflow")?;
            if *count == 0 {
                ready.insert(successor);
            }
        }
    }
    if visited != nodes.len() {
        return Err("canvas contains a connection cycle".into());
    }
    let mut rows = BTreeMap::<usize, usize>::new();
    for (id, node) in ids {
        let column = columns.get(id).copied().unwrap_or(0);
        let row = rows.entry(column).or_default();
        let position = format!("{} {} 0", 96 + column * 320, 96 + *row * 224);
        *row += 1;
        let replacement = format!("<GUILayout>{}</GUILayout>", xml::leaf("gpos", &position));
        let range = if let Some(layout) = xml::child(node, "GUILayout") {
            layout.range()
        } else {
            let end = node.range().end.checked_sub(node.tag_name().name().len() + 3).ok_or("invalid node end")?;
            end..end
        };
        edits.push(Edit { range, replacement });
    }
    Ok(nodes.len())
}
