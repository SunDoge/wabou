#![cfg(feature = "macros")]
use blockui::Record;
use blockui::RecordUpgrade;

#[derive(Record)]
struct OptionalRecord {
    title: Option<String>,
    count: Option<i64>,
    identity: Option<u64>,
    value: Option<f64>,
    enabled: Option<bool>,
}
#[test]
fn optional_fields_preserve_absence_empty_strings_and_exact_scalar_values() {
    for original in [
        OptionalRecord {
            title: None,
            count: None,
            identity: None,
            value: None,
            enabled: None,
        },
        OptionalRecord {
            title: Some(String::new()),
            count: Some(i64::MIN),
            identity: Some(u64::MAX),
            value: Some(-0.0),
            enabled: Some(false),
        },
        OptionalRecord {
            title: Some("0\n1🦀\0".into()),
            count: Some(i64::MAX),
            identity: Some(0),
            value: Some(f64::from_bits(0x7ff8000000000042)),
            enabled: Some(true),
        },
    ] {
        let decoded = OptionalRecord::decode(&original.encode().unwrap()).unwrap();
        assert_eq!(decoded.title, original.title);
        assert_eq!(decoded.count, original.count);
        assert_eq!(decoded.identity, original.identity);
        assert_eq!(
            decoded.value.map(f64::to_bits),
            original.value.map(f64::to_bits)
        );
        assert_eq!(decoded.enabled, original.enabled);
    }
    #[derive(Record)]
    struct OptionalInteger {
        value: Option<i64>,
    }
    for malformed in ["0\n", "1\n2", "2\n00", "3\n101", "2\n1x", "1\n1"] {
        assert!(OptionalInteger::decode(malformed).is_err(), "{malformed:?}");
    }
    #[derive(Record)]
    #[record(name = "optional.identity")]
    struct Text {
        value: String,
    }
    #[derive(Record)]
    #[record(name = "optional.identity")]
    struct OptionalText {
        value: Option<String>,
    }
    assert_ne!(Text::FORMAT, OptionalText::FORMAT);
    assert!(
        OptionalText::encode(&OptionalText {
            value: Some("x".repeat(1024 * 1024))
        })
        .is_err()
    );
}

#[derive(Record)]
struct NumericRecord {
    identity: u64,
    value: f64,
}
#[test]
fn unsigned_and_float_records_preserve_exact_bits() {
    for bits in [
        0,
        1 << 63,
        1,
        f64::INFINITY.to_bits(),
        f64::NEG_INFINITY.to_bits(),
        0x7ff8000000000042,
        0xfff8000000000001,
    ] {
        let original = NumericRecord {
            identity: u64::MAX,
            value: f64::from_bits(bits),
        };
        let decoded = NumericRecord::decode(&original.encode().unwrap()).unwrap();
        assert_eq!(decoded.identity, original.identity);
        assert_eq!(decoded.value.to_bits(), bits);
    }
    for malformed in [
        "2\n0116\n0000000000000000",
        "2\n-116\n0000000000000000",
        "1\n016\n7FF8000000000042",
        "1\n01\n0",
    ] {
        assert!(NumericRecord::decode(malformed).is_err());
    }
}

#[derive(Record)]
#[record(name = "app.upgrade", previous = original_location::Document)]
struct UpgradedDocument {
    title: String,
    priority: i64,
    #[record(default)]
    archived: bool,
    #[record(default = "Uncategorized".to_owned())]
    category: String,
}
#[test]
fn generated_upgrade_moves_existing_fields_and_uses_explicit_defaults() {
    let old = original_location::Document {
        title: "Draft 🦀".into(),
        priority: 42,
    };
    let buffer = old.title.as_ptr();
    let upgraded = UpgradedDocument::upgrade(old).unwrap();
    assert_eq!(upgraded.title.as_ptr(), buffer);
    assert_eq!(upgraded.priority, 42);
    assert!(!upgraded.archived);
    assert_eq!(upgraded.category, "Uncategorized");
    let decoded = UpgradedDocument::decode(&upgraded.encode().unwrap()).unwrap();
    assert_eq!(decoded.title, upgraded.title);
}

#[derive(Record, Debug, PartialEq)]
struct Document {
    title: String,
    priority: i64,
    archived: bool,
}
mod same {
    #[derive(blockui::Record)]
    pub struct Document {
        pub title: String,
        pub priority: i64,
        pub archived: bool,
    }
}
mod evolved {
    #[derive(blockui::Record)]
    pub struct Document {
        pub title: String,
        pub priority: i64,
        pub archived: bool,
        pub category: String,
    }
}
mod original_location {
    #[derive(blockui::Record)]
    #[record(name = "app.document")]
    pub struct Document {
        pub title: String,
        pub priority: i64,
    }
}
mod relocated {
    #[derive(blockui::Record)]
    #[record(name = "app.document")]
    pub struct RenamedDocument {
        pub title: String,
        pub priority: i64,
    }
    #[derive(blockui::Record)]
    #[record(name = "app.document")]
    pub struct RevisedDocument {
        pub title: String,
        pub priority: i64,
        pub archived: bool,
    }
}
#[derive(Record)]
#[record(name = "app.conditional")]
struct ConfiguredRecord {
    title: String,
    #[cfg(any())]
    excluded: Vec<String>,
}
#[derive(Record)]
#[record(name = "app.conditional")]
struct EquivalentRecord {
    title: String,
}
#[test]
fn cfg_disabled_fields_do_not_enter_codec_or_format() {
    assert_eq!(ConfiguredRecord::FORMAT, EquivalentRecord::FORMAT);
    let value = ConfiguredRecord {
        title: "active".into(),
    };
    assert_eq!(
        EquivalentRecord::decode(&value.encode().unwrap())
            .unwrap()
            .title,
        "active"
    );
}
#[test]
fn stable_record_name_survives_module_and_type_rename() {
    assert_eq!(
        original_location::Document::FORMAT,
        relocated::RenamedDocument::FORMAT
    );
    assert_ne!(
        original_location::Document::FORMAT,
        relocated::RevisedDocument::FORMAT
    );
    let original = original_location::Document {
        title: "Draft 🦀".into(),
        priority: 42,
    };
    let moved = relocated::RenamedDocument::decode(&original.encode().unwrap()).unwrap();
    assert_eq!(moved.title, original.title);
    assert_eq!(moved.priority, original.priority);
}
#[test]
fn generated_codec_round_trips_unicode_delimiters_and_integer_extremes() {
    for priority in [i64::MIN, 0, i64::MAX] {
        let value = Document {
            title: "Draft\n🦀\t\0".into(),
            priority,
            archived: true,
        };
        assert_eq!(Document::decode(&value.encode().unwrap()).unwrap(), value);
    }
    for malformed in [
        "",
        "01\nx1\n01\n0",
        "1\n🦀",
        "0\n2\n+11\n0",
        "0\n1\n01\n2",
        "0\n1\n01\n0trailing",
    ] {
        assert!(Document::decode(malformed).is_err(), "{malformed:?}");
    }
    assert!(Document::decode(&"x".repeat(1024 * 1024 + 1)).is_err());
}
#[test]
fn format_fingerprint_tracks_field_schema_without_manual_versions() {
    let fingerprint = |format: &'static str| format.split_once('@').unwrap().1;
    assert_eq!(
        fingerprint(Document::FORMAT),
        fingerprint(same::Document::FORMAT)
    );
    assert_ne!(
        fingerprint(Document::FORMAT),
        fingerprint(evolved::Document::FORMAT)
    );
    assert_ne!(Document::FORMAT, same::Document::FORMAT); // Namespace isolates independent codecs.
    assert!(Document::FORMAT.len() <= 128);
}

#[derive(Record, Debug, PartialEq)]
#[record(name = "nested.address")]
struct Address {
    city: String,
    postcode: Option<String>,
}
#[derive(Record)]
#[record(name = "nested.contact")]
struct Contact {
    name: String,
    #[record(nested)]
    address: Address,
}
#[test]
fn nested_records_preserve_unicode_boundaries_and_child_schema() {
    use blockui::record_codec::{Reader, append_record, decode_record, encode_record};
    let original = Contact {
        name: "A\n🦀\0".into(),
        address: Address {
            city: "香港\n東京".into(),
            postcode: Some(String::new()),
        },
    };
    let encoded = encode_record(&original).unwrap();
    let decoded: Contact = decode_record(&encoded).unwrap();
    assert_eq!(decoded.name, original.name);
    assert_eq!(decoded.address, original.address);
    assert_eq!(encode_record(&decoded).unwrap(), encoded);
    assert!(decode_record::<Contact>(&format!("{encoded}trailing")).is_err());
    // A rejected child decode must leave the reader at that field for migration.
    let mut child = String::new();
    append_record(&mut child, &original.address).unwrap();
    let mut reader = Reader::new(&child).unwrap();
    assert!(reader.record::<Document>().is_err());
    assert_eq!(reader.record::<Address>().unwrap(), original.address);
    reader.finish().unwrap();
}
#[test]
fn nested_record_budget_errors_preserve_parent_output() {
    let address = Address {
        city: "x".repeat(1024 * 1024),
        postcode: None,
    };
    let mut output = "existing parent".to_owned();
    assert!(blockui::record_codec::append_record(&mut output, &address).is_err());
    assert_eq!(output, "existing parent");
    let address = Address {
        city: "small".into(),
        postcode: None,
    };
    output = "x".repeat(1024 * 1024 - 1);
    let before = output.clone();
    assert!(blockui::record_codec::append_record(&mut output, &address).is_err());
    assert_eq!(output, before);
}
#[test]
fn record_envelope_budget_includes_schema_and_rejects_noncanonical_codecs() {
    struct Raw(String);
    impl Record for Raw {
        const FORMAT: &'static str = "raw.v1";
        fn encode(&self) -> blockui::Result<String> {
            Ok(self.0.clone())
        }
        fn decode(text: &str) -> blockui::Result<Self> {
            Ok(Self(text.trim().into()))
        }
    }
    use blockui::record_codec::{decode_record, encode_record};
    let exact = Raw("x".repeat(1024 * 1024 - Raw::FORMAT.len() - 1));
    let encoded = encode_record(&exact).unwrap();
    assert_eq!(encoded.len(), 1024 * 1024);
    assert!(decode_record::<Raw>(&encoded).is_ok());
    assert!(encode_record(&Raw(format!("{}x", exact.0))).is_err());
    assert!(decode_record::<Raw>("raw.v1\n padded ").is_err());
    assert!(decode_record::<Raw>("raw.v2\npayload").is_err());
}

mod nested_evolved {
    use blockui::Record;
    #[derive(Record)]
    #[record(name = "nested.address", previous = super::Address)]
    pub struct Address {
        pub city: String,
        pub postcode: Option<String>,
        #[record(default)]
        pub verified: bool,
    }
    #[derive(Record)]
    #[record(name = "nested.contact", previous = super::Contact)]
    pub struct Contact {
        pub name: String,
        #[record(nested, default = Address::upgrade(previous.address)?)]
        pub address: Address,
    }
}
#[derive(Record)]
#[record(name = "nested.wrapper")]
struct Wrapper {
    #[record(nested)]
    contact: Contact,
}
#[derive(Record)]
#[record(name = "nested.wrapper")]
struct EvolvedWrapper {
    #[record(nested)]
    contact: nested_evolved::Contact,
}
#[test]
fn nested_schema_changes_propagate_and_explicit_rust_upgrade_preserves_fields() {
    assert_ne!(Address::FORMAT, nested_evolved::Address::FORMAT);
    assert_ne!(Contact::FORMAT, nested_evolved::Contact::FORMAT);
    assert_ne!(Wrapper::FORMAT, EvolvedWrapper::FORMAT);
    let old = Contact {
        name: "Ada".into(),
        address: Address {
            city: "香港".into(),
            postcode: None,
        },
    };
    let upgraded = nested_evolved::Contact::upgrade(old).unwrap();
    assert_eq!(upgraded.name, "Ada");
    assert_eq!(upgraded.address.city, "香港");
    assert!(!upgraded.address.verified);
    let encoded = blockui::record_codec::encode_record(&upgraded).unwrap();
    assert!(blockui::record_codec::decode_record::<Contact>(&encoded).is_err());
    let decoded: nested_evolved::Contact = blockui::record_codec::decode_record(&encoded).unwrap();
    assert_eq!(decoded.address.city, "香港");
}
#[test]
fn nested_fingerprints_delimit_child_names_and_preserve_child_order() {
    use blockui::record_codec::nested_fingerprint;
    assert_ne!(
        nested_fingerprint(0, &["ab", "c"]),
        nested_fingerprint(0, &["a", "bc"])
    );
    assert_ne!(
        nested_fingerprint(0, &["a", "b"]),
        nested_fingerprint(0, &["b", "a"])
    );
}

#[derive(Record)]
#[record(name = "nested.optional")]
struct OptionalAddress {
    #[record(nested)]
    address: Option<Address>,
}
#[derive(Record)]
#[record(name = "nested.optional", previous = OptionalAddress)]
struct NewOptionalAddress {
    #[record(nested, default = previous.address.map(nested_evolved::Address::upgrade).transpose()?)]
    address: Option<nested_evolved::Address>,
}
#[test]
fn optional_child_schema_and_upgrade_are_independent_of_presence() {
    assert_ne!(OptionalAddress::FORMAT, NewOptionalAddress::FORMAT);
    for address in [
        None,
        Some(Address {
            city: "".into(),
            postcode: None,
        }),
        Some(Address {
            city: "香港\n🦀".into(),
            postcode: Some("".into()),
        }),
    ] {
        let original = OptionalAddress { address };
        let encoded = original.encode().unwrap();
        let decoded = OptionalAddress::decode(&encoded).unwrap();
        assert_eq!(decoded.address, original.address);
        let upgraded = NewOptionalAddress::upgrade(decoded).unwrap();
        assert_eq!(
            upgraded.address.as_ref().map(|address| &address.city),
            original.address.as_ref().map(|address| &address.city)
        );
        assert!(
            upgraded
                .address
                .as_ref()
                .is_none_or(|address| !address.verified)
        );
    }
    for malformed in ["0\n", "1\n2", "2\n00", "2\n10", "1\n1", "1\n0trailing"] {
        assert!(OptionalAddress::decode(malformed).is_err(), "{malformed:?}");
    }
}
#[test]
fn optional_nested_budget_error_is_atomic_and_failed_decode_can_retry() {
    use blockui::record_codec::{Reader, append_optional_record};
    let mut output = "x".repeat(1024 * 1024 - 2);
    let before = output.clone();
    assert!(append_optional_record::<Address>(&mut output, &None).is_err());
    assert_eq!(output, before);
    let mut output = String::new();
    let address = Address {
        city: "🦀".into(),
        postcode: None,
    };
    append_optional_record(&mut output, &Some(address)).unwrap();
    let mut reader = Reader::new(&output).unwrap();
    assert!(reader.optional_record::<nested_evolved::Address>().is_err());
    assert_eq!(
        reader.optional_record::<Address>().unwrap().unwrap().city,
        "🦀"
    );
    reader.finish().unwrap();
}

#[derive(Record)]
#[record(name = "nested.directory")]
struct Directory {
    #[record(nested)]
    addresses: Vec<Address>,
}
#[derive(Record)]
#[record(name = "nested.directory", previous = Directory)]
struct NewDirectory {
    #[record(nested, default = previous.addresses.into_iter().map(nested_evolved::Address::upgrade).collect::<blockui::Result<Vec<_>>>()?)]
    addresses: Vec<nested_evolved::Address>,
}
#[test]
fn nested_lists_preserve_order_empty_values_and_upgrade_every_child() {
    assert_ne!(Directory::FORMAT, NewDirectory::FORMAT);
    for addresses in [
        vec![],
        vec![
            Address {
                city: "香港\n🦀".into(),
                postcode: None,
            },
            Address {
                city: "".into(),
                postcode: Some("".into()),
            },
        ],
    ] {
        let original = Directory { addresses };
        let encoded = original.encode().unwrap();
        let decoded = Directory::decode(&encoded).unwrap();
        assert_eq!(decoded.addresses, original.addresses);
        let upgraded = NewDirectory::upgrade(decoded).unwrap();
        assert_eq!(
            upgraded
                .addresses
                .iter()
                .map(|a| &a.city)
                .collect::<Vec<_>>(),
            original
                .addresses
                .iter()
                .map(|a| &a.city)
                .collect::<Vec<_>>()
        );
        assert!(upgraded.addresses.iter().all(|a| !a.verified));
        assert!(NewDirectory::decode(&upgraded.encode().unwrap()).is_ok());
    }
}
#[test]
fn nested_lists_reject_bad_counts_truncation_schema_and_budget_atomically() {
    use blockui::record_codec::{MAX_RECORD_ITEMS, Reader, append_records};
    let mut encoded = String::new();
    append_records::<Address>(&mut encoded, &[]).unwrap();
    assert_eq!(encoded, "3\n1\n0");
    assert!(Directory::decode("3\n1\n1").is_err());
    assert!(Directory::decode("6\n4\n4097").is_err());
    assert!(Directory::decode("4\n2\n00").is_err());
    let addresses = vec![Address {
        city: "🦀".into(),
        postcode: None,
    }];
    encoded.clear();
    append_records(&mut encoded, &addresses).unwrap();
    assert!(Directory::decode(&format!("{encoded}trailing")).is_err());
    assert!(Directory::decode(&encoded[..encoded.len() - 1]).is_err());
    let mut reader = Reader::new(&encoded).unwrap();
    assert!(reader.records::<nested_evolved::Address>().is_err());
    assert_eq!(reader.records::<Address>().unwrap(), addresses);
    reader.finish().unwrap();
    let oversized: Vec<_> = (0..=MAX_RECORD_ITEMS)
        .map(|_| Address {
            city: String::new(),
            postcode: None,
        })
        .collect();
    let mut parent = "parent".to_owned();
    assert!(append_records(&mut parent, &oversized).is_err());
    assert_eq!(parent, "parent");
    let mut boundary = String::new();
    append_records(&mut boundary, &oversized[..MAX_RECORD_ITEMS]).unwrap();
    let mut reader = Reader::new(&boundary).unwrap();
    assert_eq!(reader.records::<Address>().unwrap().len(), MAX_RECORD_ITEMS);
    reader.finish().unwrap();
    let oversized = vec![Address {
        city: "x".repeat(1024 * 1024),
        postcode: None,
    }];
    assert!(append_records(&mut parent, &oversized).is_err());
    assert_eq!(parent, "parent");
}

#[derive(Record)]
#[record(name = "scalar.lists")]
struct ScalarLists {
    tags: Vec<String>,
    counts: Vec<i64>,
    identities: Vec<u64>,
    samples: Vec<f64>,
    enabled: Vec<bool>,
}
#[test]
fn scalar_record_lists_preserve_empty_strings_unicode_order_and_float_bits() {
    let original = ScalarLists {
        tags: vec![
            "".into(),
            "🦀\n\0".into(),
            "duplicate".into(),
            "duplicate".into(),
        ],
        counts: vec![i64::MIN, 0, i64::MAX],
        identities: vec![0, u64::MAX],
        samples: vec![-0.0, f64::from_bits(0x7ff8000000000042), f64::INFINITY],
        enabled: vec![false, true, false],
    };
    let encoded = original.encode().unwrap();
    let decoded = ScalarLists::decode(&encoded).unwrap();
    assert_eq!(decoded.tags, original.tags);
    assert_eq!(decoded.counts, original.counts);
    assert_eq!(decoded.identities, original.identities);
    assert_eq!(decoded.enabled, original.enabled);
    assert_eq!(
        decoded
            .samples
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>(),
        original
            .samples
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>()
    );
    assert_eq!(decoded.encode().unwrap(), encoded);
    let empty = ScalarLists {
        tags: vec![],
        counts: vec![],
        identities: vec![],
        samples: vec![],
        enabled: vec![],
    };
    assert!(ScalarLists::decode(&empty.encode().unwrap()).is_ok());
}
#[test]
fn scalar_record_list_validation_rejects_bad_counts_and_preserves_buffers() {
    use blockui::record_codec::{MAX_RECORD_ITEMS, Reader, append_fields};
    let values = vec![String::new(); MAX_RECORD_ITEMS];
    let mut output = String::new();
    append_fields(&mut output, &values).unwrap();
    let mut reader = Reader::new(&output).unwrap();
    assert_eq!(reader.fields::<String>().unwrap(), values);
    reader.finish().unwrap();
    let mut parent = "parent".to_owned();
    assert!(append_fields(&mut parent, &vec![false; MAX_RECORD_ITEMS + 1]).is_err());
    assert!(append_fields(&mut parent, &["x".repeat(1024 * 1024)]).is_err());
    assert_eq!(parent, "parent");
    for bad in ["3\n1\n1", "6\n4\n4097", "4\n2\n00", "6\n1\n11\n2"] {
        assert!(Reader::new(bad).unwrap().fields::<bool>().is_err());
    }
    let mut output = String::new();
    append_fields(&mut output, &["not boolean".to_owned()]).unwrap();
    let mut reader = Reader::new(&output).unwrap();
    assert!(reader.fields::<bool>().is_err());
    assert_eq!(reader.fields::<String>().unwrap(), ["not boolean"]);
}

#[derive(Record)]
#[record(name = "scalar.type")]
struct StringListType {
    values: Vec<String>,
}
#[derive(Record)]
#[record(name = "scalar.type")]
struct BoolListType {
    values: Vec<bool>,
}
#[test]
fn scalar_list_element_type_participates_in_schema() {
    assert_ne!(StringListType::FORMAT, BoolListType::FORMAT);
    let encoded = blockui::record_codec::encode_record(&StringListType { values: vec![] }).unwrap();
    assert!(blockui::record_codec::decode_record::<BoolListType>(&encoded).is_err());
}
