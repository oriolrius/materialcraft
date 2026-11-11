use materialcraft_core::Value;
use materialcraft_formats::sbs::{read_sbs, write_sbs};

#[test]
fn native_hurst_constants_and_functions_survive_saving() {
    for value in [
        r#"<constantValueFloat1 v="0.75"/>"#,
        r#"<dynamicValue><rootnode v="10"/><paramNodes><paramNode><uid v="10"/>
            <function v="const_float1"/><type v="256"/><funcDatas><funcData>
            <name v="const_float1"/><constantValueFloat1 v="0.75"/>
            </funcData></funcDatas></paramNode></paramNodes></dynamicValue>"#,
    ] {
        let xml = format!(
            r#"<package><content><graph><identifier v="Roughness"/><compNodes>
            <compNode><uid v="1"/><compImplementation><compFilter><filter v="fxmaps"/>
            <parameters><parameter><name v="hurst"/><paramValue>{value}</paramValue></parameter></parameters>
            </compFilter></compImplementation></compNode></compNodes></graph></content></package>"#
        );
        let mut read = read_sbs(xml.as_bytes(), "roughness.sbs", &|_| None).unwrap();
        for _ in 0..2 {
            assert!(read.warnings.is_empty(), "{:?}", read.warnings);
            let node = &read.pkg.graphs[0].nodes[0];
            if value.contains("dynamicValue") {
                let function = &node.functions["roughness"];
                assert_eq!(function.nodes.len(), 1);
                assert_eq!(function.output, Some(function.nodes[0].id));
                assert_eq!(function.nodes[0].value, Some(Value::Float(0.75)));
            } else {
                assert_eq!(node.params["roughness"], Value::Float(0.75));
            }
            let bytes = write_sbs(&read.pkg).unwrap();
            let text = std::str::from_utf8(&bytes).unwrap();
            assert!(text.contains("<name v=\"hurst\"/>"));
            assert!(!text.contains("<name v=\"roughness\"/>"));
            read = read_sbs(&bytes, "resaved.sbs", &|_| None).unwrap();
        }
    }
}
