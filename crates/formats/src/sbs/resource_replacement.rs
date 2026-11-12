//! Map unsupported resource containers and the legacy font to shared library assets.
use super::{
    library,
    sbs_reader::{SbsReader, unique_name},
    xml::{child_element, child_value},
};
use materialcraft_core::{Package, Resource, ResourceBytes};
use roxmltree::Node;

pub(super) fn target(reference: &str) -> Option<String> {
    let filename = reference.trim().rsplit(['/', '\\']).next()?;
    let replacement = if filename.eq_ignore_ascii_case("cour.ttf") {
        "cursor.otf".to_string()
    } else {
        let (stem, extension) = filename.rsplit_once('.')?;
        if !extension.eq_ignore_ascii_case("surface") {
            return None;
        }
        format!("{stem}.exr")
    };
    Some(format!("sbs://resources/{replacement}"))
}

pub(super) fn load(
    reader: &mut SbsReader<'_, '_>,
    pkg: &mut Package,
    identifier: &str,
    path: &str,
    target: &str,
    linked: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> String {
    let taken: Vec<_> = pkg.resources.iter().map(|resource| resource.name.clone()).collect();
    let name = unique_name(&taken, if identifier.is_empty() { "resource" } else { identifier });
    // A missing replacement must never fall back to the old container or embedded payload.
    let bytes =
        library::relative_path(target).ok().and_then(|_| linked(target)).filter(|bytes| bytes.len() as u64 <= crate::MAX_RESOURCE).unwrap_or_else(|| {
            reader.warnings.add(format!("resource `{path}` could not be loaded: replacement `{target}` is invalid, missing or exceeds 256 MiB"));
            Vec::new()
        });
    let mut resource = Resource::new(name.clone(), target.rsplit('/').next().unwrap_or("resource.exr"), ResourceBytes::new(bytes));
    resource.source = target.into();
    resource.folder = path.rsplit_once('/').map_or("", |(folder, _)| folder).into();
    pkg.resources.push(resource);
    name
}

pub(super) fn read_direct(reader: &mut SbsReader<'_, '_>, content: Node<'_, '_>, pkg: &mut Package, linked: &dyn Fn(&str) -> Option<Vec<u8>>) {
    let mut remaining = crate::MAX_RESOURCE as usize;
    let mut targets = std::collections::HashMap::<String, String>::new();
    for parameter in content
        .descendants()
        .filter(|node| node.has_tag_name("parameter") && matches!(child_value(*node, "name"), Some("fontdata" | "bitmapresourcepath" | "vectorresourcepath")))
    {
        let Some(reference) = child_element(parameter, "paramValue").and_then(|value| child_value(value, "constantValueString")) else { continue };
        if reference.starts_with("pkg://") || reader.direct_resource_names.contains_key(reference) {
            continue;
        }
        let Some(target) = target(reference) else { continue };
        if reader.direct_resource_names.len() >= 1024 {
            reader.warnings.add("Only the first 1024 direct resource references were read");
            break;
        }
        let name = if let Some(name) = targets.get(&target) {
            name.clone()
        } else {
            let filename = target.rsplit('/').next().unwrap_or("resource");
            let identifier = filename.rsplit_once('.').map_or(filename, |(stem, _)| stem);
            let name =
                load(reader, pkg, identifier, &format!("Resources/{identifier}"), &target, &|path| linked(path).filter(|bytes| bytes.len() <= remaining));
            if let Some(resource) = pkg.resources.last() {
                remaining = remaining.saturating_sub(resource.bytes.len());
            }
            targets.insert(target, name.clone());
            name
        };
        reader.direct_resource_names.insert(reference.into(), name);
    }
}
