#![cfg(feature = "macros")]
use blockui::RoutePath;

#[derive(Clone, Debug, Default, PartialEq, Eq, RoutePath)]
enum Destination {
    #[default]
    #[route(path = "/")]
    Home,
    #[route(path = "/projects/{project}/documents/{id}", guard = project > 0 && id > 0)]
    Document { project: u64, id: i64 },
    #[route(path = "/search/{path}/{segments}")]
    Search { path: String, segments: String },
}

#[test]
fn typed_nested_paths_round_trip_and_validate_parameters() {
    let route = Destination::Document {
        project: 12,
        id: 42,
    };
    assert_eq!(route.to_path(), "/projects/12/documents/42");
    assert_eq!(Destination::from_path(&route.to_path()), Some(route));
    assert_eq!(Destination::from_path("/"), Some(Destination::Home));
    assert_eq!(
        Destination::from_path("/search/%E6%96%87%E6%A1%A3/%F0%9F%A6%80"),
        Some(Destination::Search {
            path: "文档".into(),
            segments: "🦀".into()
        })
    );
    for invalid in [
        "",
        "//",
        "/projects/12/documents/042",
        "/projects/0/documents/42",
        "/projects/12/documents/-1",
        "/projects/12/documents/9223372036854775808",
        "/projects/12/documents/42/",
        "/projects/12/documents/42/extra",
        "/search//x",
        "/search/x%2fy/z",
        "/search/%41/z",
        "/search/%00/z",
        "/search/%FF/z",
        "/search/文档/z",
        "/search/x/z?q=1",
        "/search/x/z#fragment",
        "/search/x y/z",
    ] {
        assert_eq!(Destination::from_path(invalid), None, "{invalid}");
    }
    assert!(Destination::from_path(&format!("/search/{}/z", "x".repeat(4096))).is_none());
}

#[derive(Clone, Default, PartialEq, Eq, RoutePath)]
enum Ambiguous {
    #[default]
    #[route(path = "/")]
    Home,
    #[route(path = "/docs/new")]
    New,
    #[route(path = "/docs/{name}")]
    Document { name: String },
}
#[test]
fn overlapping_routes_are_rejected_without_source_order_precedence() {
    assert!(Ambiguous::from_path("/docs/new").is_none());
    assert!(matches!(
        Ambiguous::from_path("/docs/other"),
        Some(Ambiguous::Document { .. })
    ));
}

#[test]
fn string_parameters_encode_reserved_characters_and_decode_exactly_once() {
    let route = Destination::Search {
        path: "folder/name 🦀".into(),
        segments: "%2F?#".into(),
    };
    let path = "/search/folder%2Fname%20%F0%9F%A6%80/%252F%3F%23";
    assert_eq!(route.to_path(), path);
    assert_eq!(Destination::from_path(path), Some(route));
}

#[derive(Clone, Default, PartialEq, Eq, RoutePath)]
enum FormatOne {
    #[default]
    #[route(path = "/")]
    Home,
    #[route(path = "/items/{id}", guard = id > 0)]
    Item { id: i64 },
}
#[derive(Clone, Default, PartialEq, Eq, RoutePath)]
enum Reordered {
    #[route(path = "/items/{id}", guard = id > 0)]
    Renamed { id: i64 },
    #[default]
    #[route(path = "/")]
    Root,
}
#[derive(Clone, Default, PartialEq, Eq, RoutePath)]
enum DifferentType {
    #[default]
    #[route(path = "/")]
    Home,
    #[route(path = "/items/{id}", guard = id > 0)]
    Item { id: u64 },
}
#[derive(Clone, Default, PartialEq, Eq, RoutePath)]
enum DifferentGuard {
    #[default]
    #[route(path = "/")]
    Home,
    #[route(path = "/items/{id}", guard = id >= 0)]
    Item { id: i64 },
}
#[derive(Clone, Default, PartialEq, Eq, RoutePath)]
enum DefaultOne {
    #[default]
    #[route(path = "/")]
    Home,
    #[route(path = "/settings")]
    Settings,
}
#[derive(Clone, Default, PartialEq, Eq, RoutePath)]
enum DefaultTwo {
    #[route(path = "/")]
    Home,
    #[default]
    #[route(path = "/settings")]
    Settings,
}
#[test]
fn fingerprints_ignore_variant_names_and_order_but_track_parameter_contracts() {
    assert_eq!(FormatOne::FORMAT, Reordered::FORMAT);
    assert_ne!(FormatOne::FORMAT, DifferentType::FORMAT);
    assert_ne!(FormatOne::FORMAT, DifferentGuard::FORMAT);
    assert_ne!(FormatOne::FORMAT, Destination::FORMAT);
    assert_ne!(DefaultOne::FORMAT, DefaultTwo::FORMAT);
}

#[test]
fn pending_requests_round_trip_without_losing_replace_intent() {
    use blockui::Navigation;
    for request in [
        Navigation::Push(FormatOne::Item { id: 42 }),
        Navigation::Replace(FormatOne::Item { id: 42 }),
        Navigation::Back,
        Navigation::Forward,
    ] {
        let encoded = request.to_pending().unwrap();
        assert!(Navigation::<FormatOne>::from_pending(&encoded).unwrap() == Some(request));
    }
    assert!(Navigation::<FormatOne>::from_pending("").unwrap().is_none());
    assert_eq!(
        Navigation::Replace(FormatOne::Item { id: 42 })
            .to_pending()
            .unwrap(),
        "replace:/items/42"
    );
    for invalid in [
        "replace:",
        "replace:back",
        "replace:/items/042",
        "replace:replace:/items/42",
        "push:/items/42",
    ] {
        assert!(
            Navigation::<FormatOne>::from_pending(invalid).is_err(),
            "{invalid}"
        );
    }
}
