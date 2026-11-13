use materialcraft_core::Value;
use materialcraft_formats::sbs::{read_sbs, write_sbs};

const FALLBACK: &str = "sbs://resources/LiberationMono-Regular.ttf";

fn resource(file: &str) -> String {
    format!(r#"<package><content><resource><identifier v="font"/><type v="font"/><format v="ttf"/><filepath v="{file}"/></resource></content></package>"#)
}

fn direct(file: &str) -> String {
    format!(
        r#"<package><content><graph><identifier v="Text"/><compNodes><compNode><uid v="1"/><compImplementation><compFilter><filter v="text"/><parameters><parameter><name v="fontdata"/><paramValue><constantValueString v="{file}"/></paramValue></parameter></parameters></compFilter></compImplementation></compNode></compNodes></graph></content></package>"#
    )
}

#[test]
fn existing_relative_fonts_including_cour_take_precedence_over_the_fallback() {
    let mut empty = materialcraft_core::Package::new("Empty");
    empty.resources.push(materialcraft_core::Resource::new("font", "empty.ttf", materialcraft_core::ResourceBytes::new(Vec::new())));
    let embedded = String::from_utf8(write_sbs(&empty).unwrap()).unwrap();
    let doc = roxmltree::Document::parse(&embedded).unwrap();
    let source = &embedded[doc.descendants().find(|node| node.has_tag_name("source")).unwrap().range()];
    for file in ["fonts/cour.ttf", "fonts/custom.otf", r"fonts\Custom.TTF"] {
        for xml in [
            resource(file),
            direct(file),
            resource(file).replace("</resource>", "<source><externalcopy><filename v=\"empty.ttf\"/></externalcopy></source></resource>"),
            resource(file).replace("</resource>", &format!("{source}</resource>")),
        ] {
            let read = read_sbs(xml.as_bytes(), "example.sbs", &|path| {
                if path == "empty.ttf" {
                    return Some(Vec::new());
                }
                assert_eq!(path, file, "an available font must not read the fallback");
                Some(vec![1, 2, 3])
            })
            .unwrap();
            assert!(read.warnings.is_empty(), "{:?}", read.warnings);
            assert_eq!(read.pkg.resources.len(), 1);
            assert_eq!(read.pkg.resources[0].source, file);
            assert_eq!(read.pkg.resources[0].bytes.as_slice(), &[1, 2, 3]);
            let reopened = read_sbs(&write_sbs(&read.pkg).unwrap(), "moved.sbs", &|_| None).unwrap();
            assert!(reopened.warnings.is_empty(), "{:?}", reopened.warnings);
            assert_eq!(reopened.pkg.resources[0].bytes.as_slice(), &[1, 2, 3]);
        }
    }
}

#[test]
fn every_missing_font_uses_liberation_mono_and_survives_a_portable_save() {
    for file in ["cour.ttf", "fonts/Other.otf", "fonts/Collection.ttc", "sbs://resources/cursor.otf"] {
        for xml in [resource(file), direct(file)] {
            let read = read_sbs(xml.as_bytes(), "example.sbs", &|path| (path == FALLBACK).then(|| vec![4, 5, 6])).unwrap();
            assert!(read.warnings.is_empty(), "{:?}", read.warnings);
            assert_eq!(read.pkg.resources.len(), 1);
            let font = &read.pkg.resources[0];
            assert_eq!(font.source, FALLBACK);
            assert_eq!(font.file, "LiberationMono-Regular.ttf");
            assert_eq!(font.bytes.as_slice(), &[4, 5, 6]);
            if let Some(graph) = read.pkg.graphs.first() {
                assert_eq!(graph.nodes[0].params["fontdata"], Value::Text(font.name.clone()));
            }
            let reopened = read_sbs(&write_sbs(&read.pkg).unwrap(), "moved.sbs", &|_| None).unwrap();
            assert!(reopened.warnings.is_empty(), "{:?}", reopened.warnings);
            assert_eq!(reopened.pkg.resources[0].bytes.as_slice(), &[4, 5, 6]);

            // An unavailable fallback must keep the SBS reference repairable after saving.
            for fallback in [None, Some(Vec::new())] {
                let missing = read_sbs(xml.as_bytes(), "example.sbs", &|path| if path == FALLBACK { fallback.clone() } else { None }).unwrap();
                let original = &missing.pkg.resources[0];
                assert_eq!(original.source, file);
                assert_eq!(original.file, file.rsplit(['/', '\\']).next().unwrap());
                assert!(original.bytes.is_empty());
                assert!(!missing.warnings.is_empty());
                let saved = write_sbs(&missing.pkg).unwrap();
                let repaired = read_sbs(&saved, "example.sbs", &|path| (path == file).then(|| vec![7, 8, 9])).unwrap();
                assert!(repaired.warnings.is_empty(), "{:?}", repaired.warnings);
                assert_eq!(repaired.pkg.resources[0].name, original.name);
                assert_eq!(repaired.pkg.resources[0].bytes.as_slice(), &[7, 8, 9]);
            }
        }
    }

    for reference in ["pkg:///Resources/Original", "pkg:///Resources/Original?dependency=2", "pkg:///Resources/Original?dependency=999"] {
        let xml =
            direct(reference).replace("<package>", "<package><dependencies><dependency><filename v=\"fonts.sbs\"/><uid v=\"2\"/></dependency></dependencies>");
        for foreign in [None, Some(b"<package><content/></package>".to_vec())] {
            let read = read_sbs(xml.as_bytes(), "example.sbs", &|path| match path {
                FALLBACK => Some(vec![4, 5, 6]),
                "fonts.sbs" => foreign.clone(),
                _ => None,
            })
            .unwrap();
            assert_eq!(read.pkg.resources.len(), 1);
            let font = &read.pkg.resources[0];
            assert_eq!(font.name, "Original");
            assert_eq!(font.bytes.as_slice(), &[4, 5, 6]);
            assert_eq!(read.pkg.graphs[0].nodes[0].params["fontdata"], Value::Text(font.name.clone()));
            let reopened = read_sbs(&write_sbs(&read.pkg).unwrap(), "moved.sbs", &|_| None).unwrap();
            assert!(reopened.warnings.is_empty(), "{:?}", reopened.warnings);
            assert_eq!(reopened.pkg.resources[0].name, "Original");

            let missing = read_sbs(xml.as_bytes(), "example.sbs", &|_| None).unwrap();
            assert!(missing.pkg.resources.is_empty());
            assert_eq!(missing.pkg.graphs[0].nodes[0].params["fontdata"], Value::Text(reference.into()));
        }
    }
}
