use super::{
    Result, layout, resources,
    xml::{self, Edit},
};
use roxmltree::Node;
use std::collections::{BTreeMap, BTreeSet};

pub struct Converted {
    pub xml: String,
    pub resources: Vec<&'static str>,
    pub report: serde_json::Value,
}

fn presentation(node: Node<'_, '_>) -> bool {
    matches!(node.tag_name().name(), "GUILayout" | "GUIObjects" | "GUIName" | "desc")
        || (node.parent().is_some_and(|p| p.has_tag_name("attributes"))
            && matches!(node.tag_name().name(), "description" | "author" | "authorURL" | "icon" | "usertags"))
}

fn replaced_resource_field(node: Node<'_, '_>) -> bool {
    node.parent().is_some_and(|p| p.has_tag_name("resource")) && matches!(node.tag_name().name(), "source" | "filepath" | "format")
}

/// Compare native computational data independently of the application importer, which may not
/// understand every native field. No unsupported field may vanish through a model round trip.
pub fn semantics(document: &roxmltree::Document<'_>) -> String {
    let mut result = String::new();
    for node in document.root_element().descendants() {
        if node.ancestors().any(|n| presentation(n) || replaced_resource_field(n)) {
            continue;
        }
        if node.has_tag_name("fileUID")
            || (matches!(node.tag_name().name(), "formatVersion" | "updaterVersion") && node.parent().is_some_and(|p| p.has_tag_name("package")))
            || (node.has_tag_name("filename") && node.parent().is_some_and(|p| p.has_tag_name("dependency")))
        {
            continue;
        }
        if node.is_element() && !node.has_tag_name("attributes") {
            {
                result.push_str(&node.ancestors().filter(|n| !n.has_tag_name("attributes")).count().to_string());
                result.push('<');
                result.push_str(node.tag_name().name());
                for attribute in node.attributes() {
                    result.push(' ');
                    result.push_str(attribute.name());
                    result.push_str("=\"");
                    result.push_str(&xml::escape(attribute.value()));
                    result.push('"');
                }
                result.push('>');
            }
        } else if node.is_text() && node.text().is_some_and(|t| !t.trim().is_empty()) {
            result.push_str(node.text().unwrap_or(""));
        }
    }
    result
}

fn description(owner: Node<'_, '_>) -> String {
    let identifier = xml::value(owner, "identifier").or_else(|| xml::value(owner, "name")).unwrap_or("value");
    let name = xml::title(identifier);
    if matches!(owner.tag_name().name(), "graph" | "function") {
        let inputs: Vec<_> = xml::child(owner, "paraminputs")
            .into_iter()
            .flat_map(|n| n.children())
            .filter(Node::is_element)
            .filter_map(|n| xml::value(n, "identifier"))
            .collect();
        let outputs: Vec<_> = xml::child(owner, "graphOutputs")
            .into_iter()
            .flat_map(|n| n.children())
            .filter(Node::is_element)
            .filter_map(|n| xml::value(n, "identifier"))
            .collect();
        let kind = if owner.has_tag_name("function") { "reusable value function" } else { "processing graph" };
        format!(
            "{name}: {kind}. Inputs: {}. Outputs: {}.",
            if inputs.is_empty() { "none".into() } else { inputs.join(", ") },
            if outputs.is_empty() { "declared function result".into() } else { outputs.join(", ") }
        )
    } else {
        format!("Controls the {name} value in this graph.")
    }
}

fn attributes(owner: Node<'_, '_>, text: &str, description: &str) -> Result<String> {
    let mut output = String::from("<attributes>");
    if let Some(attributes) = xml::child(owner, "attributes") {
        for child in attributes.children().filter(Node::is_element).filter(|n| !presentation(*n)) {
            output.push_str(text.get(child.range()).ok_or("invalid attribute range")?);
        }
    }
    output.push_str(&xml::leaf("description", description));
    output.push_str("</attributes>");
    Ok(output)
}

pub fn package(text: &str, name: &str, identities: &BTreeMap<String, String>) -> Result<Converted> {
    let document = xml::parse(text)?;
    let root = document.root_element();
    if !root.has_tag_name("package") {
        return Err("expected a native SBS package".into());
    }
    let before = semantics(&document);
    let mut edits = Vec::new();
    for tag in ["formatVersion", "updaterVersion"] {
        if xml::child(root, tag).is_none() {
            let offset = root.first_element_child().ok_or("package has no content")?.range().start;
            edits.push(Edit { range: offset..offset, replacement: xml::leaf(tag, env!("CARGO_PKG_VERSION")) });
        }
    }
    let mut resource_keys = Vec::new();
    let mut graphs = 0;
    let mut functions = 0;
    let mut nodes = 0;
    let mut canvases = 0;
    let mut dependencies = 0;
    for node in root.descendants().filter(Node::is_element) {
        if node.ancestors().skip(1).any(|n| n.has_tag_name("GUIObjects") || n.has_tag_name("resource") || n.has_tag_name("attributes")) {
            continue;
        }
        match node.tag_name().name() {
            "graph" => graphs += 1,
            "function" if node.parent().is_some_and(|p| p.has_tag_name("content")) => functions += 1,
            "paramNodes" | "compNodes" | "paramsGraphNodes" => {
                nodes += layout::reposition(node, &mut edits).map_err(|e| format!("{name}: {e}"))?;
                canvases += 1;
            }
            "attributes" => {
                let owner = node.parent().ok_or("attribute owner is missing")?;
                edits.push(Edit { range: node.range(), replacement: attributes(owner, text, &description(owner))? });
            }
            "GUIObjects" => edits.push(Edit { range: node.range(), replacement: String::new() }),
            "GUIName" => xml::set(&mut edits, node, ""),
            "desc" => {
                let owner = node.parent().ok_or("description owner is missing")?;
                let identifier = xml::value(owner, "identifier").unwrap_or("library");
                xml::set(&mut edits, node, &format!("{} reusable graph collection.", xml::title(identifier)));
            }
            "fileUID" if node.parent().is_some_and(|p| p.has_tag_name("package")) => {
                xml::set(&mut edits, node, identities.get(name).ok_or("missing package identity")?);
            }
            "formatVersion" | "updaterVersion" if node.parent().is_some_and(|p| p.has_tag_name("package")) => {
                xml::set(&mut edits, node, env!("CARGO_PKG_VERSION"));
            }
            "dependency" => {
                let filename = xml::child(node, "filename").ok_or("dependency has no filename")?;
                let reference = filename.attribute("v").ok_or("dependency has no filename value")?;
                let target = if reference == "?himself" {
                    name
                } else {
                    dependencies += 1;
                    reference.strip_prefix("sbs://").ok_or_else(|| format!("{name}: unexpected dependency scheme"))?
                };
                let identity = identities.get(target).ok_or_else(|| format!("{name}: unresolved package {target}"))?;
                if reference != "?himself" {
                    xml::set(&mut edits, filename, &xml::relative(name, target)?);
                }
                if let Some(uid) = xml::child(node, "fileUID") {
                    xml::set(&mut edits, uid, identity);
                }
            }
            "resource" => {
                let key = xml::value(node, "identifier").ok_or("resource has no identifier")?;
                let recipe = resources::recipe(key)?;
                if let Some(original) = xml::value(node, "filepath").and_then(|path| path.rsplit('/').next()).filter(|name| !name.is_empty()) {
                    let output = std::path::Path::new(recipe.path);
                    if std::path::Path::new(original).file_stem() != output.file_stem() {
                        return Err(format!("{name}: resource `{key}` would change its filename stem").into());
                    }
                    let extension = std::path::Path::new(original).extension();
                    // Surface inputs are replaced with a standard image container;
                    // every other resource keeps its original filename and extension.
                    if extension != Some(std::ffi::OsStr::new("surface")) && extension != output.extension() {
                        return Err(format!("{name}: resource `{key}` would change its extension").into());
                    }
                }
                let mut replacement = String::from("<resource>");
                for child in node.children().filter(Node::is_element) {
                    if !matches!(child.tag_name().name(), "source" | "filepath" | "format" | "attributes") {
                        replacement.push_str(text.get(child.range()).ok_or("invalid resource range")?);
                    }
                }
                replacement.push_str(&xml::leaf("format", recipe.extension));
                replacement.push_str(&xml::leaf("filepath", &xml::relative(name, recipe.path)?));
                replacement.push_str(&attributes(node, text, recipe.description)?);
                replacement.push_str("</resource>");
                edits.push(Edit { range: node.range(), replacement });
                resource_keys.push(recipe.key);
            }
            _ => {}
        }
    }
    let output = xml::apply(text, edits)?;
    let after = xml::parse(&output)?;
    if semantics(&after) != before {
        return Err(format!("{name}: conversion changed native computational data").into());
    }
    if after.descendants().any(|n| n.has_tag_name("binembedded") || n.has_tag_name("icon")) {
        return Err(format!("{name}: source asset payload survived conversion").into());
    }
    Ok(Converted {
        xml: output,
        resources: resource_keys,
        report: serde_json::json!({
            "package": name, "graphs": graphs, "functions": functions, "canvases": canvases,
            "positioned_nodes": nodes, "dependencies": dependencies, "native_semantics_preserved": true,
        }),
    })
}

pub fn verify_references(text: &str, owner: &str, available: &BTreeSet<String>) -> Result<()> {
    let document = xml::parse(text)?;
    for node in document.descendants().filter(Node::is_element) {
        let is_dependency = node.has_tag_name("filename") && node.parent().is_some_and(|p| p.has_tag_name("dependency"));
        let is_resource = node.has_tag_name("filepath") && node.parent().is_some_and(|p| p.has_tag_name("resource"));
        if !is_dependency && !is_resource {
            continue;
        }
        let reference = node.attribute("v").ok_or("missing reference value")?;
        if reference == "?himself" {
            continue;
        }
        if reference.contains("://") {
            return Err("converted references must be package-relative".into());
        }
        let resolved = materialcraft_formats::sbs::library::resolve(&format!("sbs://{owner}"), reference)?;
        let relative = materialcraft_formats::sbs::library::relative_path(&resolved)?;
        if !available.contains(relative) {
            return Err(format!("{owner}: reference does not resolve: {reference}").into());
        }
        if is_resource && !relative.starts_with("resources/") {
            return Err("resource is outside the shared resources folder".into());
        }
    }
    Ok(())
}
