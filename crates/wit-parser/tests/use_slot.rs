use wit_parser::*;

#[test]
fn parse_use_slot_world() {
    let wit = r#"
package local:demos;

interface types {
    resource greeting {
        constructor(name: string);
        greet: func(salutation: string) -> string;
    }
}

interface greeter {
    use types.{greeting};
    make-greeter: func(name: string) -> greeting;
    greet: func(g: borrow<greeting>, salutation: string) -> string;
}

world proxy {
    use types1: types;
    use types2: types;
    import in1: greeter with { types1 as types };
    import in2: greeter with { types2 as types };
    export out1: greeter with { types1 as types };
    export out2: greeter with { types2 as types };
}
"#;

    let group = UnresolvedPackageGroup::parse("test.wit", wit).unwrap();
    let mut resolve = Resolve::new();
    let pkg_id = resolve.push_group(group).unwrap();

    for (_name, world_id) in &resolve.packages[pkg_id].worlds {
        let world = &resolve.worlds[*world_id];

        assert_eq!(world.use_slots.len(), 2);
        assert!(world.use_slots.contains_key("types1"));
        assert!(world.use_slots.contains_key("types2"));

        let in1 = world.imports.get(&WorldKey::Name("in1".into())).unwrap();
        if let WorldItem::Interface { with, .. } = in1 {
            assert_eq!(with.len(), 1);
            assert_eq!(with[0].slot_name, "types1");
            assert_eq!(with[0].use_target, "types");
        } else {
            panic!("in1 should be Interface");
        }

        let out1 = world.exports.get(&WorldKey::Name("out1".into())).unwrap();
        if let WorldItem::Interface { with, .. } = out1 {
            assert_eq!(with.len(), 1);
            assert_eq!(with[0].slot_name, "types1");
            assert_eq!(with[0].use_target, "types");
        } else {
            panic!("out1 should be Interface");
        }

        let slot = &world.use_slots["types1"];
        if let WorldItem::UseSlot { id, with, .. } = slot {
            let iface = &resolve.interfaces[*id];
            assert_eq!(iface.name.as_deref(), Some("types"));
            assert!(with.is_empty(), "use slot should not have with entries in this test");
        } else {
            panic!("types1 should be UseSlot");
        }
    }
}
