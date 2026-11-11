use super::{Result, xml};
use std::collections::BTreeSet;

pub fn function_paths(document: &roxmltree::Document<'_>) -> BTreeSet<String> {
    document
        .descendants()
        .filter(|node| node.has_tag_name("function") && node.parent().is_some_and(|parent| parent.has_tag_name("content")))
        .filter_map(|node| {
            let name = xml::value(node, "identifier")?;
            let mut parts: Vec<_> =
                node.ancestors().filter(|ancestor| ancestor.has_tag_name("group")).filter_map(|group| xml::value(group, "identifier")).collect();
            parts.reverse();
            parts.push(name);
            Some(parts.join("/"))
        })
        .collect()
}

/// Make bare standard-function calls portable using their already declared package dependency.
/// Local functions and explicit references retain precedence; unknown targets are never guessed.
pub fn qualify(text: &str, standard: &BTreeSet<String>) -> Result<(String, usize)> {
    let document = xml::parse(text)?;
    let local = function_paths(&document);
    let dependencies: Vec<_> = xml::child(document.root_element(), "dependencies")
        .into_iter()
        .flat_map(|node| node.children())
        .filter(|node| node.has_tag_name("dependency") && xml::value(*node, "filename") == Some("sbs://functions.sbs"))
        .filter_map(|node| xml::value(node, "uid"))
        .collect();
    let mut edits = Vec::new();
    for node in document.descendants().filter(|node| node.has_tag_name("paramNode") && xml::value(*node, "function") == Some("instance")) {
        for value in xml::child(node, "funcDatas")
            .into_iter()
            .flat_map(|node| node.children())
            .filter(|node| node.has_tag_name("funcData") && xml::value(*node, "name") == Some("instance"))
            .flat_map(|node| node.descendants())
            .filter(|node| node.has_tag_name("constantValueString"))
        {
            let Some(path) = value.attribute("v").and_then(|value| value.strip_prefix("pkg:///")) else { continue };
            if local.contains(path) || !standard.contains(path) {
                continue;
            }
            let [uid] = dependencies.as_slice() else { return Err("bare standard function requires one declared functions.sbs dependency".into()) };
            xml::set(&mut edits, value, &format!("pkg:///{path}?dependency={uid}"));
        }
    }
    let count = edits.len();
    Ok((xml::apply(text, edits)?, count))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn only_known_nonlocal_function_calls_gain_the_declared_dependency() {
        let body = r#"<group><identifier v="Functions"/><content><group><identifier v="Random"/><content>
            <function><identifier v="Sample_[A,B["/></function></content></group></content></group>"#;
        let library = format!("<package><content>{body}</content></package>");
        let standard = function_paths(&xml::parse(&library).unwrap());
        assert_eq!(standard, BTreeSet::from(["Functions/Random/Sample_[A,B[".into()]));
        let source = r#"<package><dependencies><dependency><filename v="sbs://functions.sbs"/><uid v="9"/></dependency></dependencies>
            <content><graph><identifier v="Example"/><parameters><parameter><paramValue><dynamicValue><paramNodes><paramNode>
            <function v="instance"/><funcDatas><funcData><name v="instance"/><constantValue><constantValueString v="pkg:///Functions/Random/Sample_[A,B["/>
            </constantValue></funcData></funcDatas></paramNode></paramNodes></dynamicValue></paramValue></parameter></parameters></graph></content></package>"#;
        let (qualified, count) = qualify(source, &standard).unwrap();
        assert_eq!(count, 1);
        assert_eq!(qualified, source.replace("pkg:///Functions/Random/Sample_[A,B[", "pkg:///Functions/Random/Sample_[A,B[?dependency=9"));
        let explicit = source.replace("Sample_[A,B[", "Sample_[A,B[?dependency=7");
        assert_eq!(qualify(&explicit, &standard).unwrap(), (explicit, 0));
        let unknown = source.replace("Sample_[A,B[", "Unknown");
        assert_eq!(qualify(&unknown, &standard).unwrap(), (unknown, 0));
        let local = source.replacen("<content>", &format!("<content>{body}"), 1);
        assert_eq!(qualify(&local, &standard).unwrap(), (local, 0));
        assert!(qualify(&source.replace("sbs://functions.sbs", "sbs://other.sbs"), &standard).is_err());
    }
}
