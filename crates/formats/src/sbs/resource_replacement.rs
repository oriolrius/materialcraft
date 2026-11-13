//! Map unsupported resource containers and missing fonts to shared library assets.
use super::{
    library,
    sbs_reader::{SbsReader, unique_name},
    xml::{child_element, child_value},
};
use materialcraft_core::{Package, Resource, ResourceBytes};
use roxmltree::Node;

pub(super) fn target(reference: &str) -> Option<String> {
    let filename = reference.trim().rsplit(['/', '\\']).next()?;
    let (stem, extension) = filename.rsplit_once('.')?;
    extension.eq_ignore_ascii_case("surface").then(|| format!("sbs://resources/{stem}.exr"))
}

pub(super) fn is_font(reference: &str) -> bool {
    reference.rsplit_once('.').is_some_and(|(_, extension)| matches!(extension.to_ascii_lowercase().as_str(), "ttf" | "otf" | "ttc" | "otc" | "woff" | "woff2"))
}

fn store(pkg: &mut Package, identifier: &str, path: &str, source: &str, bytes: Vec<u8>) -> String {
    let taken: Vec<_> = pkg.resources.iter().map(|resource| resource.name.clone()).collect();
    let name = unique_name(&taken, if identifier.is_empty() { "resource" } else { identifier });
    let mut resource = Resource::new(name.clone(), crate::safe_file_name(source.rsplit(['/', '\\']).next().unwrap_or("resource")), ResourceBytes::new(bytes));
    resource.source = source.into();
    resource.folder = path.rsplit_once('/').map_or("", |(folder, _)| folder).into();
    pkg.resources.push(resource);
    name
}

pub(super) fn load(
    reader: &mut SbsReader<'_, '_>,
    pkg: &mut Package,
    identifier: &str,
    path: &str,
    target: &str,
    linked: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> String {
    // A missing replacement must never fall back to the old container or embedded payload.
    let bytes =
        library::relative_path(target).ok().and_then(|_| linked(target)).filter(|bytes| bytes.len() as u64 <= crate::MAX_RESOURCE).unwrap_or_else(|| {
            reader.warnings.add(format!("resource `{path}` could not be loaded: replacement `{target}` is invalid, missing or exceeds 256 MiB"));
            Vec::new()
        });
    store(pkg, identifier, path, target, bytes)
}

pub(super) fn load_font(pkg: &mut Package, identifier: &str, path: &str, linked: &dyn Fn(&str) -> Option<Vec<u8>>) -> Option<String> {
    let bytes = linked(library::FALLBACK_FONT).filter(|bytes| !bytes.is_empty() && bytes.len() as u64 <= crate::MAX_RESOURCE)?;
    Some(store(pkg, identifier, path, library::FALLBACK_FONT, bytes))
}

pub(super) fn read_direct(reader: &mut SbsReader<'_, '_>, content: Node<'_, '_>, pkg: &mut Package, linked: &dyn Fn(&str) -> Option<Vec<u8>>) {
    let mut remaining = crate::MAX_RESOURCE as usize;
    let mut targets = std::collections::HashMap::<String, String>::new();
    for parameter in content
        .descendants()
        .filter(|node| node.has_tag_name("parameter") && matches!(child_value(*node, "name"), Some("fontdata" | "bitmapresourcepath" | "vectorresourcepath")))
    {
        let Some(reference) = child_element(parameter, "paramValue").and_then(|value| child_value(value, "constantValueString")) else { continue };
        if reference.trim().is_empty() || reader.direct_resource_names.contains_key(reference) {
            continue;
        }
        let font = child_value(parameter, "name") == Some("fontdata");
        let package_reference = reference.starts_with("pkg://");
        if package_reference && (!font || reader.resource_name(reference).is_some()) {
            continue;
        }
        if reader.direct_resource_names.len() >= 1024 {
            reader.warnings.add("Only the first 1024 direct resource references were read");
            break;
        }
        if font {
            let original = if package_reference { reader.resolve_reference(reference).0 } else { reference.to_string() };
            let filename = original.rsplit(['/', '\\']).next().unwrap_or("font");
            let identifier = if package_reference { filename } else { filename.rsplit_once('.').map_or(filename, |(stem, _)| stem) };
            let path = format!("Resources/{identifier}");
            let supplied = if package_reference { None } else { linked(reference).filter(|bytes| !bytes.is_empty() && bytes.len() <= remaining) };
            let name = if let Some(bytes) = supplied {
                store(pkg, identifier, &path, reference, bytes)
            } else if let Some(name) = load_font(pkg, identifier, &path, &|path| linked(path).filter(|bytes| bytes.len() <= remaining)) {
                name
            } else {
                reader.warnings.add(format!("font `{reference}` and fallback `{}` are unavailable", library::FALLBACK_FONT));
                // Without a declaration there is no filename to invent: keep the unresolved SBS reference.
                if package_reference {
                    reader.direct_resource_names.insert(reference.into(), reference.into());
                    continue;
                }
                store(pkg, identifier, &path, reference, Vec::new())
            };
            if let Some(resource) = pkg.resources.last() {
                remaining = remaining.saturating_sub(resource.bytes.len());
            }
            reader.direct_resource_names.insert(reference.into(), name);
            continue;
        }
        let Some(target) = target(reference) else { continue };
        let name = if let Some(name) = targets.get(&target) {
            name.clone()
        } else {
            let filename = target.rsplit(['/', '\\']).next().unwrap_or("resource");
            let identifier = filename.rsplit_once('.').map_or(filename, |(stem, _)| stem);
            let path = format!("Resources/{identifier}");
            let name = load(reader, pkg, identifier, &path, &target, &|path| linked(path).filter(|bytes| bytes.len() <= remaining));
            if let Some(resource) = pkg.resources.last() {
                remaining = remaining.saturating_sub(resource.bytes.len());
            }
            targets.insert(target, name.clone());
            name
        };
        reader.direct_resource_names.insert(reference.into(), name);
    }
}
