use wit_encoder::packages_from_parsed;

const WIT: &str = r#"package local:demos;

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

/// Verify the encoder correctly handles UseSlot and `with` entries
/// by checking structural equivalence after round-tripping.
#[test]
fn round_trip() {
    let mut resolve = wit_parser::Resolve::new();
    resolve.push_str("", WIT).unwrap();
    let packages = packages_from_parsed(&resolve);
    assert_eq!(packages.len(), 1);
    let encoded = packages[0].to_string();

    // Parse the encoded output to verify it's valid WIT
    let mut resolve2 = wit_parser::Resolve::new();
    resolve2.push_str("", &encoded).unwrap();
    let pkg_id = *resolve2.package_names.values().next().unwrap();
    let world_id = *resolve2.packages[pkg_id].worlds.values().next().unwrap();
    let world2 = &resolve2.worlds[world_id];

    // Verify the round-tripped world has the correct structure
    assert_eq!(world2.use_slots.len(), 2, "should have 2 use slots");
    assert!(world2.use_slots.contains_key("types1"), "should have types1");
    assert!(world2.use_slots.contains_key("types2"), "should have types2");

    let in1 = world2.imports.get(&wit_parser::WorldKey::Name("in1".into())).unwrap();
    let out1 = world2.exports.get(&wit_parser::WorldKey::Name("out1".into())).unwrap();
    match (in1, out1) {
        (
            wit_parser::WorldItem::Interface { with: w1, .. },
            wit_parser::WorldItem::Interface { with: w2, .. },
        ) => {
            assert_eq!(w1.len(), 1, "in1 should have 1 with entry");
            assert_eq!(w1[0].slot_name, "types1");
            assert_eq!(w1[0].use_target, "types");
            assert_eq!(w2.len(), 1, "out1 should have 1 with entry");
            assert_eq!(w2[0].slot_name, "types1");
            assert_eq!(w2[0].use_target, "types");
        }
        _ => panic!("in1/out1 should be Interface"),
    }

    // Second round-trip: encoder → string → parse → encoder → string
    let packages2 = packages_from_parsed(&resolve2);
    let encoded2 = packages2[0].to_string();
    let mut resolve3 = wit_parser::Resolve::new();
    resolve3.push_str("", &encoded2).unwrap();
    let pkg_id3 = *resolve3.package_names.values().next().unwrap();
    let world_id3 = *resolve3.packages[pkg_id3].worlds.values().next().unwrap();
    let world3 = &resolve3.worlds[world_id3];

    assert_eq!(world3.use_slots.len(), 2);
    assert!(world3.use_slots.contains_key("types1"));
    assert!(world3.use_slots.contains_key("types2"));
}
