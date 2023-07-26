use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::exit;
use uuid::Uuid;
use anytree_plugin_cargo::load_dependencies;

const SBOM_DEFAULT_FILE_NAME: &str = "sbom.spdx.json";
const CONFIG_FILE: &str = "Gosh.yaml";
const CARGO_PLUGIN_FLAG: &str = "cargo plugin";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    test_build().await
}

async fn test_build() -> anyhow::Result<()> {
    // init trace
    anytree_utils::tracing::default_init();

    // parse url arg
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        eprintln!("First arg should be url");
        exit(1);
    }
    let context: GitContext = args[1].parse()?;
    tracing::trace!("Context: {:?}", context);

    //  gosh://0:b00a7a5a24740e4a7d6487d31969732f1febcaea412df5cc307400818055ad58/anytree-devtest/anytree-test-project
    let project_name = context
        .remote
        .clone()
        .split('/')
        .last()
        .ok_or(anyhow::format_err!("Malformed url"))?
        .to_string();
    tracing::trace!("Project name: {}", project_name);

    // prepare `run_dir`
    let mut run_dir = dirs::cache_dir()
        .unwrap_or(PathBuf::from(".cache"))
        .join("gosh")
        .join("builder")
        .join(format!("run_dir_{}", Uuid::new_v4().to_string()))
        .join(&project_name);
    tracing::trace!("Run dir: {:?}", run_dir);

    // load_dependencies()
    // // checkout repo
    // let cache = GitCacheRepo::with_dir(context.remote, run_dir.clone());
    // cache.update().await?;
    // cache.checkout(context.git_ref).await?;

    // // check Gosh.yaml
    // run_dir.push(CONFIG_FILE);
    // tracing::trace!("Gosh.yaml path: {:?}", run_dir);
    // assert!(run_dir.exists());
    // let config = File::open(&run_dir)?;
    // let reader = BufReader::new(config);
    // run_dir.pop();
    //
    // for line in reader.lines() {
    //     if let Ok(line) = line {
    //         if line.contains(CARGO_PLUGIN_FLAG) {
    //             tracing::trace!("Start load of cargo dependencies");
    //             let mut sbom_path = run_dir.clone();
    //             sbom_path.push(SBOM_DEFAULT_FILE_NAME);
    //             tracing::trace!("SBON path: {:?}", sbom_path);
    //             assert!(sbom_path.exists());
    //             let mut registry_path = run_dir.clone();
    //             registry_path.push(".gosh");
    //             registry_path.push("cargo");
    //             registry_path.push("registry");
    //             std::fs::create_dir_all(&registry_path)?;
    //             tracing::trace!("Registry path: {:?}", registry_path);
    //
    //             load_dependencies_from_sbom(
    //                 sbom_path.to_str().unwrap(),
    //                 registry_path.to_str().unwrap(),
    //             )
    //             .await?;
    //         }
    //     }
    // }
    // tracing::trace!("Run docker build");
    // // docker build -t anytree_test .
    // // docker run -it --rm --name anytree_test_run anytree_test
    // tracing::trace!("docker build -t {} .", &project_name);
    // let status = Command::new("docker")
    //     .arg("build")
    //     .arg("-t")
    //     .arg(&project_name)
    //     .arg(".")
    //     .current_dir(&run_dir)
    //     .status()
    //     .await?;
    //
    // if !status.success() {
    //     anyhow::bail!("Failed to build docker");
    // }
    //
    // tracing::trace!("Run docker");
    // tracing::trace!(
    //     "docker run -it --rm --name {} {}.",
    //     &project_name,
    //     &project_name
    // );
    // let status = Command::new("docker")
    //     .arg("run")
    //     .arg("-it")
    //     .arg("--rm")
    //     .arg("--name")
    //     .arg(&project_name)
    //     .arg(&project_name)
    //     .current_dir(&run_dir)
    //     .status()
    //     .await?;
    //
    // if !status.success() {
    //     anyhow::bail!("Failed to build docker");
    // }

    Ok(())
}
