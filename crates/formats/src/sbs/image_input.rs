use materialcraft_nodes::{MAX_IMAGE_INPUTS, image_input_prefix};

pub(super) fn read(definition: &str, identifier: &str) -> Option<String> {
    let prefix = image_input_prefix(definition)?;

    // Native FX-Map sockets use inputpattern; accept the earlier writer's input spelling too.
    let native_identifier = if definition == "fx_map" { identifier.strip_prefix("inputpattern").map(|suffix| format!("input{suffix}")) } else { None };
    let identifier = native_identifier.as_deref().unwrap_or(identifier);
    let index = if identifier == "input" { 0 } else { identifier.strip_prefix("input:")?.parse::<usize>().ok()? };
    (index < MAX_IMAGE_INPUTS).then(|| format!("{prefix}{index}"))
}

pub(super) fn write(definition: &str, identifier: &str) -> Option<String> {
    let prefix = image_input_prefix(definition)?;

    let index = identifier.strip_prefix(prefix)?.parse::<usize>().ok()?;
    let native = if definition == "fx_map" { "inputpattern" } else { "input" };
    (index < MAX_IMAGE_INPUTS).then(|| if index == 0 { native.into() } else { format!("{native}:{index}") })
}
