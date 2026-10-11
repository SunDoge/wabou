//! Public developer commands; legacy xtask forwards here.
use blockui_dev::{
    Options,
    project::{Project, checked},
};
use clap::{Args, CommandFactory, FromArgMatches, Parser, Subcommand};
use std::{
    path::PathBuf,
    process::{Command, ExitCode},
};
mod scaffold;
mod static_native;
mod test;
mod web;
mod web_hot;
#[derive(Parser)]
#[command(
    name = "blockui",
    version,
    about = "Develop native and browser BlockUI applications"
)]
struct Cli {
    #[command(subcommand)]
    command: Task,
}
#[derive(Args, Default)]
struct ProjectArgs {
    /// Cargo package name, app directory or Cargo.toml. Defaults to the current project.
    app: Option<String>,
    #[arg(short, long, conflicts_with = "app")]
    package: Option<String>,
    #[arg(long = "manifest-path", alias = "manifest")]
    manifest: Option<PathBuf>,
    #[arg(long, default_value = "")]
    features: String,
    #[arg(long)]
    target_dir: Option<PathBuf>,
}
impl ProjectArgs {
    // An explicit package selector can be validated by Cargo itself. Paths and
    // default-app selection still need project discovery.
    fn package_command(&self, operation: &str) -> Option<Command> {
        // Explicit manifests retain discovery's project-directory/config behavior.
        if self.manifest.is_some() {
            return None;
        }
        let name = self.package.as_deref().or(self.app.as_deref())?;
        let path = std::path::Path::new(name);
        if path.components().count() != 1
            || !matches!(
                path.components().next(),
                Some(std::path::Component::Normal(_))
            )
            || path.is_file()
            || (self.manifest.is_none() && path.is_dir())
        {
            return None;
        }
        let mut command = blockui_dev::project::cargo_command();
        command.arg(operation).arg("--package").arg(name);
        if let Some(target) = &self.target_dir {
            command.arg("--target-dir").arg(target);
        }
        if !self.features.is_empty() {
            command.arg("--features").arg(&self.features);
        }
        Some(command)
    }
    fn project(&self, web: bool) -> Result<Project, String> {
        Project::discover(
            self.package.as_deref().or(self.app.as_deref()),
            self.manifest.as_deref(),
            self.target_dir.as_deref(),
            &self.features,
            web,
        )
    }
    fn options(self, project: &Project) -> Options {
        Options {
            manifest: Some(project.manifest.clone()),
            features: self.features,
            target_dir: Some(project.target.clone()),
            ..Default::default()
        }
    }
}
#[derive(Args)]
struct LaunchArgs {
    /// Build/run the application binary with a linked UI instead of a dylib host.
    #[arg(long = "static", conflicts_with_all = ["web", "host_package", "task", "prepare_only", "configuration_only"])]
    static_native: bool,
    #[command(flatten)]
    project: ProjectArgs,
    #[arg(long)]
    web: bool,
    #[arg(long, default_value_t = 8123)]
    port: u16,
    #[arg(long)]
    host_package: Option<String>,
    #[arg(long)]
    capture: bool,
    #[arg(long)]
    task: Option<String>,
    #[arg(long)]
    configuration_only: bool,
    #[arg(long)]
    prepare_only: bool,
    #[arg(long)]
    release: bool,
}
#[derive(Subcommand)]
enum Task {
    /// Create a standalone Rust application with native and Wasm entries.
    New {
        path: PathBuf,
        #[arg(long)]
        sdk_path: Option<PathBuf>,
        /// Use the prototype wabou-native SDK and testing namespace.
        #[arg(long)]
        wabou_native: bool,
        /// Override the native facade crate location for framework integrations.
        #[arg(long, requires = "wabou_native")]
        native_sdk_path: Option<PathBuf>,
    },
    /// Build and run with native or browser hot reload.
    Dev(LaunchArgs),
    /// Build and run once, without source watching.
    Run(LaunchArgs),
    /// Build the native host and plugin, or static browser assets.
    Build {
        #[command(flatten)]
        project: ProjectArgs,
        #[arg(long)]
        web: bool,
        #[arg(long)]
        release: bool,
        #[arg(long)]
        host_package: Option<String>,
        #[arg(long = "static", conflicts_with_all = ["web", "host_package"])]
        static_native: bool,
    },
    /// Type-check the application library and optionally native component tests.
    Check {
        #[command(flatten)]
        project: ProjectArgs,
        #[arg(long)]
        web: bool,
        /// Check the linked native binary and its required features, without linking.
        #[arg(long = "static", conflicts_with_all = ["web", "tests", "test"])]
        static_native: bool,
        /// Check library, binary unit and integration tests without linking or running them.
        #[arg(long, conflicts_with_all = ["web", "test"])]
        tests: bool,
        /// Check a selected native integration target; repeat for multiple targets.
        #[arg(long = "test", conflicts_with = "web")]
        test: Vec<String>,
        /// Cargo diagnostic format, e.g. json or short; validated by Cargo.
        #[arg(long)]
        message_format: Option<String>,
        /// Write Cargo's HTML build timing report in the target directory.
        #[arg(long)]
        timings: bool,
    },
    /// Run application unit/component tests without building a native host or Wasm release.
    Test {
        #[command(flatten)]
        project: ProjectArgs,
        /// Select an integration test target; repeat for multiple targets.
        #[arg(long = "test")]
        tests: Vec<String>,
        /// Run only library unit tests (can combine with --test).
        #[arg(long)]
        lib: bool,
        /// Match test names, e.g. --filter toast.
        #[arg(long)]
        filter: Option<String>,
        /// Compile selected tests without running them.
        #[arg(long)]
        no_run: bool,
        /// Write Cargo's HTML report for test compilation.
        #[arg(long)]
        timings: bool,
        /// Test harness arguments, e.g. -- --nocapture.
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Check Cargo, rustfmt and application configuration; --web checks the Wasm toolchain.
    Doctor {
        #[command(flatten)]
        project: ProjectArgs,
        #[arg(long)]
        web: bool,
        /// Validate a linked native binary instead of a cdylib target.
        #[arg(long = "static", conflicts_with = "web")]
        static_native: bool,
    },
}
fn launch(args: LaunchArgs, watch: bool) -> Result<(), String> {
    if args.static_native {
        if watch {
            return Err(
                "static native UI does not use dylib hot reload; use run/build --static".into(),
            );
        }
        let project = args.project.project(false)?;
        return static_native::launch(
            &project,
            &args.project.features,
            args.release,
            args.capture,
            true,
        );
    }
    let project = args.project.project(args.web)?;
    if args.web {
        if args.capture
            || args.task.is_some()
            || args.host_package.is_some()
            || args.prepare_only
            || args.configuration_only
        {
            return Err("native launch options are unavailable with --web".into());
        }
        let assets = web::build(&project, &args.project.features)?;
        return web::serve(
            &assets,
            args.port,
            if watch {
                Some(crate::web_hot::Dev::start(
                    project,
                    args.project.features,
                    &assets,
                )?)
            } else {
                None
            },
        );
    }
    let mut options = args.project.options(&project);
    options.host_package = args.host_package;
    options.capture = args.capture;
    options.task = args.task;
    options.configuration_only = args.configuration_only;
    options.prepare_only = args.prepare_only;
    options.release = args.release;
    blockui_dev::launch_project(&project, watch, options)
}
fn run(cli: Cli, command_name: &str) -> Result<(), String> {
    match cli.command {
        Task::New {
            path,
            sdk_path,
            wabou_native,
            native_sdk_path,
        } => scaffold::create(
            &path,
            sdk_path.as_deref(),
            wabou_native,
            native_sdk_path.as_deref(),
            command_name,
        ),
        Task::Dev(args) => launch(args, true),
        Task::Run(args) => launch(args, false),
        Task::Build {
            project,
            web,
            release,
            host_package,
            static_native,
        } => {
            let p = project.project(web)?;
            if static_native {
                return static_native::launch(&p, &project.features, release, false, false);
            }
            if web {
                if host_package.is_some() {
                    return Err("--host-package is native-only".into());
                }
                web::build(&p, &project.features).map(|_| ())
            } else {
                let mut options = project.options(&p);
                options.release = release;
                options.build_only = true;
                options.host_package = host_package;
                blockui_dev::launch_project(&p, false, options)
            }
        }
        Task::Check {
            project,
            web,
            static_native,
            tests,
            test,
            message_format,
            timings,
        } => {
            let mut command = if static_native {
                let p = project.project(false)?;
                static_native::command(&p, &project.features, "check")?.0
            } else if !web && let Some(command) = project.package_command("check") {
                command
            } else {
                let p = if !web {
                    Project::discover_for_tests(
                        project.package.as_deref().or(project.app.as_deref()),
                        project.manifest.as_deref(),
                        project.target_dir.as_deref(),
                        &project.features,
                    )?
                } else {
                    project.project(web)?
                };
                p.cargo("check", &project.features)
            };
            if !static_native {
                command.arg("--lib");
            }
            if tests {
                command.arg("--tests");
            }
            for target in test {
                command.arg("--test").arg(target);
            }
            if web {
                command.args(["--target", "wasm32-unknown-unknown"]);
            }
            if let Some(format) = message_format {
                command.arg("--message-format").arg(format);
            }
            if timings {
                command.arg("--timings");
            }
            checked(&mut command)
        }
        Task::Test {
            project,
            tests,
            lib,
            filter,
            no_run,
            timings,
            args,
        } => {
            let options = test::Options {
                tests: &tests,
                lib,
                filter: filter.as_deref(),
                no_run,
                timings,
                args: &args,
            };
            if (lib || !tests.is_empty())
                && let Some(command) = project.package_command("test")
            {
                return checked(&mut test::selected(command, options));
            }
            let p = Project::discover_for_tests(
                project.package.as_deref().or(project.app.as_deref()),
                project.manifest.as_deref(),
                project.target_dir.as_deref(),
                &project.features,
            )?;
            checked(&mut test::command(&p, &project.features, options)?)
        }
        Task::Doctor {
            project,
            web,
            static_native,
        } => {
            let p = project.project(web)?;
            checked(blockui_dev::project::cargo_command().arg("--version"))?;
            checked(blockui_dev::project::rustc_command().arg("--version"))?;
            checked(Command::new("rustfmt").arg("--version"))?;
            if static_native {
                let (_, binary) = static_native::command(&p, &project.features, "check")?;
                println!("Native binary: {binary}");
            } else if web {
                web::doctor(&p)?;
                let assets = web::assets(&p)?;
                if !assets.join("index.html").is_file() {
                    return Err("browser asset directory needs index.html".into());
                }
            } else {
                blockui_dev::configuration(
                    &p.metadata,
                    &p.manifest,
                    p.name(),
                    &p.target,
                    &project.features,
                )?;
            }
            println!("OK: {} ({})", p.name(), p.package_root().display());
            Ok(())
        }
    }
}
pub fn main() -> ExitCode {
    main_from(std::env::args_os())
}

/// Reuse the native developer command parser from a parent framework CLI.
/// The first argument is the command name, as with ordinary process arguments.
pub fn main_from<I, T>(arguments: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    main_from_named("blockui", arguments)
}

/// Embed the parser with a parent command name in help and diagnostics.
/// Arguments still include the first executable-name argument.
pub fn main_from_named<I, T>(command_name: &'static str, arguments: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let matches = Cli::command()
        .bin_name(command_name)
        .get_matches_from(arguments);
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    match run(cli, command_name) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
