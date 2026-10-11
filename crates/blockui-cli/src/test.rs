//! Keep the default feedback loop out of renderer and deployment builds.
use blockui_dev::project::Project;
use std::process::Command;
#[derive(Clone, Copy, Default)]
pub(super) struct Options<'a> {
    pub(super) tests: &'a [String],
    pub(super) lib: bool,
    pub(super) filter: Option<&'a str>,
    pub(super) no_run: bool,
    pub(super) timings: bool,
    pub(super) args: &'a [String],
}
pub(super) fn command(
    project: &Project,
    features: &str,
    options: Options<'_>,
) -> Result<Command, String> {
    let Options { tests, lib, .. } = options;
    let mut command = project.cargo("test", features);
    if tests.is_empty() && !lib {
        command.arg("--lib");
        if let Some(targets) = project.package["targets"].as_array() {
            let gated = targets.iter().any(|target| {
                target["kind"]
                    .as_array()
                    .is_some_and(|kinds| kinds.iter().any(|kind| kind == "test"))
                    && target["test"].as_bool().unwrap_or(true)
                    && target["required-features"]
                        .as_array()
                        .is_some_and(|features| !features.is_empty())
            });
            // Cargo owns feature resolution, including workspace unification.
            // Pay for the dependency graph only when a target needs it.
            let resolved = if gated {
                Some(Project::discover(
                    Some(project.name()),
                    Some(&project.manifest),
                    Some(&project.target),
                    features,
                    false,
                )?)
            } else {
                None
            };
            let nodes = resolved
                .as_ref()
                .and_then(|resolved| resolved.metadata["resolve"]["nodes"].as_array());
            let application = nodes.and_then(|nodes| {
                nodes
                    .iter()
                    .find(|node| node["id"] == project.package["id"])
            });
            if gated && application.is_none() {
                return Err("Cargo metadata did not report application features".into());
            }
            for target in targets {
                let available = target["required-features"]
                    .as_array()
                    .is_none_or(|required| {
                        required.iter().all(|feature| {
                            feature.as_str().is_some_and(|feature| {
                                application.zip(nodes).is_some_and(|(application, nodes)| {
                                    feature_enabled(application, nodes, feature)
                                })
                            })
                        })
                    });
                let integration = target["kind"]
                    .as_array()
                    .is_some_and(|kinds| kinds.iter().any(|kind| kind.as_str() == Some("test")));
                if integration
                    && available
                    && target["test"].as_bool().unwrap_or(true)
                    && let Some(name) = target["name"].as_str()
                {
                    command.arg("--test").arg(name);
                }
            }
        }
    } else {
        return Ok(selected(command, options));
    }
    Ok(finish(command, options))
}

pub(super) fn selected(mut command: Command, options: Options<'_>) -> Command {
    let Options { tests, lib, .. } = options;
    if lib {
        command.arg("--lib");
    }
    for test in tests {
        command.arg("--test").arg(test);
    }
    finish(command, options)
}

fn finish(mut command: Command, options: Options<'_>) -> Command {
    let Options {
        filter,
        no_run,
        timings,
        args,
        ..
    } = options;
    if timings {
        command.arg("--timings");
    }
    if no_run {
        command.arg("--no-run");
    }
    if let Some(filter) = filter.filter(|filter| filter.starts_with('-')) {
        // End both Cargo and harness option parsing before a literal filter.
        command.arg("--").args(args).arg("--").arg(filter);
    } else {
        if let Some(filter) = filter {
            command.arg(filter);
        }
        if !args.is_empty() {
            command.arg("--").args(args);
        }
    }
    command
}

// Cargo's node.features lists local names. Dependency features are on the
// resolved dependency node, looked up by dependency alias rather than package name.
fn feature_enabled(
    application: &serde_json::Value,
    nodes: &[serde_json::Value],
    feature: &str,
) -> bool {
    let (node, name) = if let Some((dependency, name)) = feature.split_once('/') {
        let dependency = dependency.replace('-', "_");
        let Some(edge) = application["deps"].as_array().and_then(|deps| {
            deps.iter()
                .find(|edge| edge["name"].as_str() == Some(&dependency))
        }) else {
            return false;
        };
        let Some(node) = nodes.iter().find(|node| node["id"] == edge["pkg"]) else {
            return false;
        };
        (node, name)
    } else {
        (application, feature)
    };
    node["features"].as_array().is_some_and(|features| {
        features
            .iter()
            .any(|feature| feature.as_str() == Some(name))
    })
}

#[cfg(test)]
mod tests {
    use super::feature_enabled;
    use serde_json::json;
    #[test]
    fn disabled_gated_tests_do_not_resolve_the_dependency_graph() {
        let project = blockui_dev::project::Project {
            metadata: json!({}),
            package: json!({
                "name": "fixture",
                "targets": [{
                    "name": "manual-proof", "kind": ["test"],
                    "test": false, "required-features": ["native"]
                }]
            }),
            // Discovery would fail: no Cargo process is needed for a target
            // that the default test selection will never run.
            manifest: "nonexistent-blockui-fixture/Cargo.toml".into(),
            root: ".".into(),
            target: "target".into(),
        };
        let command = super::command(&project, "", super::Options::default())
            .expect("disabled tests must not trigger discovery");
        let args: Vec<_> = command.get_args().collect();
        assert!(args.contains(&std::ffi::OsStr::new("--lib")));
        assert!(!args.contains(&std::ffi::OsStr::new("--test")));
    }
    #[test]
    fn leading_hyphen_filter_is_literal_after_harness_options() {
        let mut cargo = std::process::Command::new("cargo");
        cargo.arg("test");
        let command = super::selected(
            cargo,
            super::Options {
                lib: true,
                filter: Some("--ignored"),
                timings: true,
                args: &["--test-threads=1".into()],
                ..Default::default()
            },
        );
        let args: Vec<_> = command
            .get_args()
            .map(|arg| arg.to_str().unwrap())
            .collect();
        assert_eq!(
            args,
            [
                "test",
                "--lib",
                "--timings",
                "--",
                "--test-threads=1",
                "--",
                "--ignored"
            ]
        );
    }
    #[test]
    fn required_features_use_resolved_dependency_aliases() {
        let application =
            json!({"features":["local"], "deps":[{"name":"renamed_dep", "pkg":"dependency-id"}]});
        let nodes = [json!({"id":"dependency-id", "features":["enabled"]})];
        assert!(feature_enabled(&application, &nodes, "local"));
        assert!(feature_enabled(&application, &nodes, "renamed-dep/enabled"));
        assert!(!feature_enabled(
            &application,
            &nodes,
            "renamed-dep/disabled"
        ));
        assert!(!feature_enabled(&application, &nodes, "missing/enabled"));
        assert!(!feature_enabled(&application, &nodes, "disabled"));
    }
}
