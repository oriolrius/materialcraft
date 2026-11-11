use materialcraft_core::Value;
use materialcraft_formats::sbs::{read_sbs, write_sbs};

fn package() -> String {
    let node = |uid, dependency| {
        format!(
            r#"<compNode><uid v="{uid}"/><compImplementation><compFilter><filter v="bitmap"/><parameters>
        <parameter><name v="bitmapresourcepath"/><paramValue><constantValueString v="pkg:///Resources/tile?dependency={dependency}"/></paramValue></parameter>
        </parameters></compFilter></compImplementation></compNode>"#
        )
    };
    format!(
        r#"<package><dependencies><dependency><filename v="?himself"/><uid v="1"/></dependency>
        <dependency><filename v="nested/source.sbs"/><uid v="2"/></dependency></dependencies><content>
        <group><identifier v="Resources"/><content><resource><identifier v="tile"/><type v="bitmap"/><format v="png"/>
        <filepath v="resources/local.png"/></resource></content></group>
        <graph><identifier v="Example"/><compNodes>{}{}{}</compNodes></graph></content></package>"#,
        node(1, 1),
        node(2, 2),
        node(3, 2)
    )
}

const SOURCE: &[u8] = br#"<package><content><group><identifier v="Resources"/><content>
    <resource><identifier v="tile"/><type v="bitmap"/><format v="png"/><filepath v="../resources/tile.png"/></resource>
    </content></group></content></package>"#;

#[test]
fn foreign_resource_uses_its_owner_without_shadowing_local_names_and_round_trips() {
    let reads = std::cell::RefCell::new(Vec::new());
    let read = read_sbs(package().as_bytes(), "caller.sbs", &|path| {
        reads.borrow_mut().push(path.to_string());
        match path {
            "nested/source.sbs" => Some(SOURCE.to_vec()),
            "resources/local.png" => Some(vec![1, 2, 3]),
            "nested/../resources/tile.png" => Some(vec![4, 5, 6]),
            _ => None,
        }
    })
    .unwrap();
    assert!(read.warnings.is_empty(), "{:?}", read.warnings);
    assert_eq!(reads.borrow().iter().filter(|path| *path == "nested/source.sbs").count(), 1);
    assert_eq!(read.pkg.resources.len(), 2);
    let names: Vec<_> = read.pkg.graphs[0]
        .nodes
        .iter()
        .map(|node| match &node.params["resource"] {
            Value::Text(name) => name.as_str(),
            _ => panic!("resource should be text"),
        })
        .collect();
    assert_ne!(names[0], names[1]);
    assert_eq!(names[1], names[2]);
    assert_eq!(read.pkg.resource(names[0]).unwrap().bytes.as_slice(), &[1, 2, 3]);
    let foreign = read.pkg.resource(names[1]).unwrap();
    assert_eq!(foreign.bytes.as_slice(), &[4, 5, 6]);
    assert_eq!(foreign.source, "nested/../resources/tile.png");
    let saved = write_sbs(&read.pkg).unwrap();
    let reopened = read_sbs(&saved, "relocated.sbs", &|_| None).unwrap();
    assert!(reopened.warnings.is_empty(), "{:?}", reopened.warnings);
    assert_eq!(reopened.pkg.resource(names[1]).unwrap().bytes.as_slice(), &[4, 5, 6]);
    assert_eq!(reopened.pkg.graphs[0].nodes[1].params["resource"], Value::Text(names[1].into()));
}

#[test]
fn missing_or_invalid_foreign_resource_never_falls_back_to_a_local_name() {
    for source in [None, Some(b"not XML".to_vec()), Some(b"<package><content/></package>".to_vec())] {
        let read = read_sbs(package().as_bytes(), "caller.sbs", &|path| {
            if path == "resources/local.png" {
                Some(vec![1])
            } else if path == "nested/source.sbs" {
                source.clone()
            } else {
                None
            }
        })
        .unwrap();
        assert!(!read.warnings.is_empty());
        assert!(!read.pkg.graphs[0].nodes[1].params.contains_key("resource"));
        assert_eq!(read.pkg.resources.len(), 1);
    }
    let unknown = package().replace("?dependency=2", "?dependency=99");
    let read = read_sbs(unknown.as_bytes(), "caller.sbs", &|path| (path == "resources/local.png").then(|| vec![1])).unwrap();
    assert!(read.warnings.iter().any(|warning| warning.contains("unknown dependency")));
    assert!(!read.pkg.graphs[0].nodes[1].params.contains_key("resource"));
}
