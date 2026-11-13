#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use std::collections::{BTreeMap, BTreeSet};

const PACKAGE: &str = r#"<?xml version="1.0"?><package><identifier v="Example"/><fileUID v="old"/>
<formatVersion v="1.1.0.202502"/><dependencies><dependency><filename v="?himself"/><uid v="1"/><fileUID v="old"/></dependency>
<dependency><filename v="sbs://functions.sbs"/><uid v="2"/><fileUID v="old-functions"/></dependency></dependencies><content>
<graph><identifier v="process"/><attributes><description v="Old prose"/><author v="Old author"/><icon v="payload"/><physicalSize v="1 2 3"/></attributes>
<compNodes><compNode><uid v="10"/><GUILayout><gpos v="1 2 3"/><docked v="1"/></GUILayout><compImplementation><futureOperation v="retain"/></compImplementation></compNode>
<compNode><uid v="20"/><connections><connection><identifier v="input"/><connRef v="10"/><connRefOutput v="out"/></connection></connections></compNode></compNodes>
<GUIObjects><GUIObject><type v="COMMENT"/><description v="Old annotation"/></GUIObject></GUIObjects>
<parameters><parameter><name v="gain"/><paramValue><dynamicValue><rootnode v="30"/><paramNodes>
<paramNode><uid v="30"/><function v="future_function"/><funcDatas><funcData><name v="future"/><constantValueString v="retain verbatim"/></funcData></funcDatas></paramNode>
</paramNodes></dynamicValue></paramValue></parameter></parameters></graph>
<group><identifier v="Resources"/><content><resource><identifier v="DoorFrame"/><uid v="40"/><type v="svg"/><format v="SVG"/>
<filepath v="sbs://"/><source><binembedded><strdata v="not even valid base64"/></binembedded></source></resource></content></group>
</content></package>"#;

fn identities(name: &str) -> BTreeMap<String, String> {
    BTreeMap::from([(name.into(), xml::identity(name)), ("functions.sbs".into(), "0".into())])
}

#[test]
fn conversion_starts_both_versions_at_the_workspace_release() {
    let saved = materialcraft_formats::sbs::write_sbs(&materialcraft_core::Package::new("Version check")).unwrap();
    let saved = xml::parse(std::str::from_utf8(&saved).unwrap()).unwrap();
    let expected = xml::value(saved.root_element(), "updaterVersion").unwrap();
    for source in [PACKAGE.to_string(), PACKAGE.replacen("<content>", "<updaterVersion v=\"1.1.0.202502\"/><content>", 1)] {
        let converted = convert::package(&source, "example.sbs", &identities("example.sbs")).unwrap();
        let converted = xml::parse(&converted.xml).unwrap();
        assert_eq!(xml::value(converted.root_element(), "updaterVersion"), Some(expected));
        assert_eq!(xml::value(converted.root_element(), "formatVersion"), Some(expected));
        assert_eq!(converted.root_element().children().filter(|n| n.has_tag_name("updaterVersion")).count(), 1);
    }
}

#[test]
fn native_conversion_keeps_unknown_computations_and_replaces_embedded_art_without_decoding_it() {
    let owner = "materials/pbr/example.sbs";
    let converted = convert::package(PACKAGE, owner, &identities(owner)).unwrap();
    assert_eq!(convert::semantics(&xml::parse(PACKAGE).unwrap()), convert::semantics(&xml::parse(&converted.xml).unwrap()));
    for retained in ["futureOperation", "future_function", "retain verbatim", "physicalSize v=\"1 2 3\""] {
        assert!(converted.xml.contains(retained));
    }
    for removed in ["Old prose", "Old author", "payload", "Old annotation", "binembedded", "sbs://", "docked"] {
        assert!(!converted.xml.contains(removed), "{removed}");
    }
    assert!(converted.xml.contains("../../resources/DoorFrame.svg"));
    assert!(converted.xml.contains("../../functions.sbs"));
    let document = xml::parse(&converted.xml).unwrap();
    assert_eq!(document.descendants().filter(|n| n.has_tag_name("gpos")).count(), 3);
    assert_eq!(converted.xml, convert::package(PACKAGE, owner, &identities(owner)).unwrap().xml);
    let available = BTreeSet::from([owner.into(), "functions.sbs".into(), "resources/DoorFrame.svg".into()]);
    convert::verify_references(&converted.xml, owner, &available).unwrap();
}

#[test]
fn unknown_assets_missing_dependencies_and_broken_graphs_are_errors() {
    let ids = identities("example.sbs");
    assert!(convert::package(&PACKAGE.replace("DoorFrame", "unknown_asset"), "example.sbs", &ids).is_err());
    assert!(convert::package(&PACKAGE.replace("sbs://functions.sbs", "sbs://absent.sbs"), "example.sbs", &ids).is_err());
    assert!(convert::package(&PACKAGE.replace("connRef v=\"10\"", "connRef v=\"99\""), "example.sbs", &ids).is_err());
    let cycle = PACKAGE.replace("<compImplementation>", "<connections><connection><connRef v=\"20\"/></connection></connections><compImplementation>");
    assert!(convert::package(&cycle, "example.sbs", &ids).is_err());
}

#[test]
fn references_are_relative_portable_and_confined_to_the_library() {
    assert_eq!(xml::relative("materials/pbr/a.sbs", "resources/a.png").unwrap(), "../../resources/a.png");
    assert_eq!(xml::relative("materials/pbr/a.sbs", "materials/pbr/b.sbs").unwrap(), "b.sbs");
    assert!(xml::relative("a.sbs", "../private.png").is_err());
    let converted = convert::package(PACKAGE, "example.sbs", &identities("example.sbs")).unwrap();
    assert!(convert::verify_references(&converted.xml, "example.sbs", &BTreeSet::new()).is_err());
    let escaped = converted.xml.replace("resources/DoorFrame.svg", "../outside.svg");
    assert!(convert::verify_references(&escaped, "example.sbs", &BTreeSet::new()).is_err());
}

#[test]
fn resource_names_survive_and_surface_replacements_declare_their_actual_formats() {
    for (key, original, expected, format) in [
        ("courier_new", "cour.ttf", "LiberationMono-Regular.ttf", "ttf"),
        ("moss01_bmp", "moss01_bmp.bmp", "moss01_bmp.bmp", "bmp"),
        ("bluenoise_256_c", "bluenoise_256_c.surface", "bluenoise_256_c.exr", "exr"),
        ("shuf4_64_32", "shuf4_64_32.surface", "shuf4_64_32.exr", "exr"),
        ("sun", "sun.surface", "sun.exr", "exr"),
    ] {
        let xml = format!("<package><content><resource><identifier v=\"{key}\"/><filepath v=\"sbs://resources/{original}\"/></resource></content></package>");
        let output = convert::package(&xml, "example.sbs", &identities("example.sbs")).unwrap();
        assert!(output.xml.contains(&format!("<filepath v=\"resources/{expected}\"/>")));
        assert!(output.xml.contains(&format!("<format v=\"{format}\"/>")));
        assert!(convert::package(&xml.replace(original, "different.png"), "example.sbs", &identities("example.sbs")).is_err());
    }
}

#[test]
fn generated_permutation_has_uniform_ranks_and_blue_noise_has_the_expected_shape() {
    let permutation = image::load_from_memory(&resources::load("perm").unwrap()).unwrap().into_luma16();
    assert_eq!(permutation.dimensions(), (1024, 16));
    for row in permutation.as_raw().as_chunks::<1024>().0.iter() {
        assert_eq!(row.iter().collect::<BTreeSet<_>>().len(), 1024);
        assert_eq!(row.iter().copied().min(), Some(0));
        assert_eq!(row.iter().copied().max(), Some(u16::MAX));
    }
    let noise = resources::load("bluenoise_256").unwrap();
    assert_eq!(noise, resources::load("bluenoise_256").unwrap());
    let noise = image::load_from_memory(&noise).unwrap().into_luma16();
    assert_eq!(noise.dimensions(), (256, 256));
}

#[test]
fn library_art_is_decodable_without_reading_source_assets() {
    for key in ["DoorFrame", "scratched_metal", "leather", "scratch_damage", "moss01_bmp", "sun", "bluenoise_256_c"] {
        let bytes = resources::load(key).unwrap();
        let image = materialcraft_formats::decode(&bytes, true).unwrap();
        assert!(image.w > 0 && image.h > 0);
        assert!(image.data.iter().all(|value| value.is_finite()));
        let min = image.data.iter().copied().fold(f32::INFINITY, f32::min);
        let max = image.data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!(max > min, "constant replacement {key}");
    }
}

#[test]
fn curated_resources_keep_the_supplied_bytes_and_required_notices() {
    for key in ["bluenoise_256", "bluenoise_256_c", "sun", "shuf4_64_32", "courier_new", "scratch_damage", "leather", "scratched_metal", "moss01_bmp"] {
        let recipe = resources::recipe(key).unwrap();
        if key == "courier_new" && materialcraft_formats::sbs::library::read(&format!("sbs://{}", recipe.path)).is_err() {
            assert!(resources::load(key).is_err(), "a build without craft-fonts cannot export its font");
            continue;
        }
        let bytes = resources::load(key).unwrap();
        let supplied = materialcraft_formats::sbs::library::read(&format!("sbs://{}", recipe.path)).unwrap();
        assert!(bytes == supplied, "converter replaced the supplied bytes for {key}");
        assert!(!resources::notices(key).is_empty(), "missing license notice for {key}");
        for notice in resources::notices(key) {
            assert!(!materialcraft_formats::sbs::library::read(&format!("sbs://resources/{notice}")).unwrap().is_empty());
        }
    }
    if let Ok(font) = resources::load("courier_new") {
        assert!(font.starts_with(b"\0\x01\0\0"));
    }
    let shuffled = image::load_from_memory(&resources::load("shuf4_64_32").unwrap()).unwrap().into_rgba32f();
    assert_eq!(shuffled.dimensions(), (64, 32));
    assert!(shuffled.as_raw().iter().all(|v| v.is_finite()));
}
