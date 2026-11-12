//! Import referenced resource declarations without recursively importing their owner's graphs.
use super::{
    constant::{MAX_XML_NODES, SELF_DEPENDENCY},
    sbs_reader::SbsReader,
    xml::{child_element, child_value},
};
use materialcraft_core::Package;
use roxmltree::Node;
use std::collections::{BTreeMap, BTreeSet};

fn relative_to(owner: &str, file: &str) -> String {
    let file = file.replace('\\', "/");
    if file.contains(':') || file.starts_with('/') {
        return file;
    }
    let owner = owner.replace('\\', "/");
    owner.rsplit_once('/').map_or_else(|| file.clone(), |(folder, _)| format!("{folder}/{file}"))
}

fn resource_path(resource: Node<'_, '_>) -> String {
    let mut parts: Vec<_> = resource.ancestors().skip(1).filter(|node| node.has_tag_name("group")).filter_map(|node| child_value(node, "identifier")).collect();
    parts.reverse();
    parts.push(child_value(resource, "identifier").unwrap_or(""));
    parts.join("/")
}

pub(super) fn read(reader: &mut SbsReader<'_, '_>, content: Node<'_, '_>, pkg: &mut Package, linked: &dyn Fn(&str) -> Option<Vec<u8>>) {
    let mut needed: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for parameter in content
        .descendants()
        .filter(|node| node.has_tag_name("parameter") && matches!(child_value(*node, "name"), Some("bitmapresourcepath" | "vectorresourcepath" | "fontdata")))
    {
        let Some(url) = child_element(parameter, "paramValue").and_then(|value| child_value(value, "constantValueString")) else { continue };
        let (path, dependency) = reader.resolve_reference(url);
        if let Some(file) = dependency.filter(|file| file != SELF_DEPENDENCY) {
            needed.entry(file).or_default().insert(path);
        }
    }
    // Dependency resources cannot trigger recursive graph loads or unbounded XML work.
    let mut remaining = 64 * 1024 * 1024_usize;
    let mut remaining_data = crate::MAX_RESOURCE as usize;
    if needed.len() > 64 {
        reader.warnings.add("Only the first 64 resource dependencies were read");
    }
    for (file, paths) in needed.into_iter().take(64) {
        let result = (|| -> Result<(), String> {
            let bytes = linked(&file).ok_or("source package is unavailable")?;
            remaining = remaining.checked_sub(bytes.len()).ok_or("linked resource XML exceeds 64 MiB")?;
            let text = std::str::from_utf8(&bytes).map_err(|_| "source package is not UTF-8")?;
            let document = roxmltree::Document::parse_with_options(text, roxmltree::ParsingOptions { allow_dtd: false, nodes_limit: MAX_XML_NODES })
                .map_err(|_| "source package is not valid XML")?;
            if !document.root_element().has_tag_name("package") {
                return Err("source is not an SBS package".into());
            }
            for resource in document.descendants().filter(|node| node.has_tag_name("resource")) {
                let path = resource_path(resource);
                if !paths.contains(&path) || reader.external_resource_names.contains_key(&(file.clone(), path.clone())) {
                    continue;
                }
                if reader.external_resource_names.len() >= 1024 {
                    return Err("linked resources exceed the 1024-resource limit".into());
                }
                let identifier = child_value(resource, "identifier").unwrap_or("resource");
                let name = reader.read_resource(resource, identifier, &path, pkg, &|reference| {
                    linked(&relative_to(&file, reference)).filter(|bytes| bytes.len() <= remaining_data)
                });
                if let Some(imported) = pkg.resources.last_mut() {
                    if !imported.source.is_empty() {
                        imported.source = relative_to(&file, &imported.source);
                    }
                    if let Some(left) = remaining_data.checked_sub(imported.bytes.len()) {
                        remaining_data = left;
                    } else {
                        imported.bytes = materialcraft_core::ResourceBytes::new(Vec::new());
                        reader.warnings.add("Linked resource data exceeds 256 MiB; remaining data is unavailable");
                    }
                }
                reader.external_resource_names.insert((file.clone(), path), name);
            }
            Ok(())
        })();
        if let Err(error) = result {
            reader.warnings.add(format!("Could not load resources from `{file}`: {error}"));
        }
    }
}
