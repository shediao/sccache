#![cfg(all(feature = "dist-client", feature = "dist-server"))]

extern crate assert_cmd;
#[macro_use]
extern crate log;
extern crate ccache;
extern crate serde_json;

use crate::harness::{
    get_stats, ccache_command, start_local_daemon, stop_local_daemon, write_json_cfg, write_source,
};
use assert_cmd::prelude::*;
use ccache::config::HTTPUrl;
use ccache::dist::{
    AssignJobResult, CompileCommand, InputsReader, JobId, JobState, RunJobResult, ServerIncoming,
    ServerOutgoing, SubmitToolchainResult, Toolchain, ToolchainReader,
};
use std::ffi::OsStr;
use std::path::Path;

use ccache::errors::*;

mod harness;

fn basic_compile(tmpdir: &Path, ccache_cfg_path: &Path, ccache_cached_cfg_path: &Path) {
    let envs: Vec<(_, &OsStr)> = vec![
        ("RUST_BACKTRACE", "1".as_ref()),
        ("CCACHE_LOG", "debug".as_ref()),
        ("CCACHE_CONF", ccache_cfg_path.as_ref()),
        ("CCACHE_CACHED_CONF", ccache_cached_cfg_path.as_ref()),
    ];
    let source_file = "x.c";
    let obj_file = "x.o";
    write_source(tmpdir, source_file, "#if !defined(CCACHE_TEST_DEFINE)\n#error CCACHE_TEST_DEFINE is not defined\n#endif\nint x() { return 5; }");
    ccache_command()
        .args(&[
            std::env::var("CC")
                .unwrap_or_else(|_| "gcc".to_string())
                .as_str(),
            "-c",
            "-DCCACHE_TEST_DEFINE",
        ])
        .arg(tmpdir.join(source_file))
        .arg("-o")
        .arg(tmpdir.join(obj_file))
        .envs(envs)
        .assert()
        .success();
}

pub fn dist_test_ccache_client_cfg(
    tmpdir: &Path,
    scheduler_url: HTTPUrl,
) -> ccache::config::FileConfig {
    let mut ccache_cfg = harness::ccache_client_cfg(tmpdir);
    ccache_cfg.cache.disk.as_mut().unwrap().size = 0;
    ccache_cfg.dist.scheduler_url = Some(scheduler_url);
    ccache_cfg
}

#[test]
#[cfg_attr(not(feature = "dist-tests"), ignore)]
fn test_dist_basic() {
    let tmpdir = tempfile::Builder::new()
        .prefix("ccache_dist_test")
        .tempdir()
        .unwrap();
    let tmpdir = tmpdir.path();
    let ccache_dist = harness::ccache_dist_path();

    let mut system = harness::DistSystem::new(&ccache_dist, tmpdir);
    system.add_scheduler();
    system.add_server();

    let ccache_cfg = dist_test_ccache_client_cfg(tmpdir, system.scheduler_url());
    let ccache_cfg_path = tmpdir.join("ccache-cfg.json");
    write_json_cfg(tmpdir, "ccache-cfg.json", &ccache_cfg);
    let ccache_cached_cfg_path = tmpdir.join("ccache-cached-cfg");

    stop_local_daemon();
    start_local_daemon(&ccache_cfg_path, &ccache_cached_cfg_path);
    basic_compile(tmpdir, &ccache_cfg_path, &ccache_cached_cfg_path);

    get_stats(|info| {
        assert_eq!(1, info.stats.dist_compiles.values().sum::<usize>());
        assert_eq!(0, info.stats.dist_errors);
        assert_eq!(1, info.stats.compile_requests);
        assert_eq!(1, info.stats.requests_executed);
        assert_eq!(0, info.stats.cache_hits.all());
        assert_eq!(1, info.stats.cache_misses.all());
    });
}

#[test]
#[cfg_attr(not(feature = "dist-tests"), ignore)]
fn test_dist_restartedserver() {
    let tmpdir = tempfile::Builder::new()
        .prefix("ccache_dist_test")
        .tempdir()
        .unwrap();
    let tmpdir = tmpdir.path();
    let ccache_dist = harness::ccache_dist_path();

    let mut system = harness::DistSystem::new(&ccache_dist, tmpdir);
    system.add_scheduler();
    let server_handle = system.add_server();

    let ccache_cfg = dist_test_ccache_client_cfg(tmpdir, system.scheduler_url());
    let ccache_cfg_path = tmpdir.join("ccache-cfg.json");
    write_json_cfg(tmpdir, "ccache-cfg.json", &ccache_cfg);
    let ccache_cached_cfg_path = tmpdir.join("ccache-cached-cfg");

    stop_local_daemon();
    start_local_daemon(&ccache_cfg_path, &ccache_cached_cfg_path);
    basic_compile(tmpdir, &ccache_cfg_path, &ccache_cached_cfg_path);

    system.restart_server(&server_handle);
    basic_compile(tmpdir, &ccache_cfg_path, &ccache_cached_cfg_path);

    get_stats(|info| {
        assert_eq!(2, info.stats.dist_compiles.values().sum::<usize>());
        assert_eq!(0, info.stats.dist_errors);
        assert_eq!(2, info.stats.compile_requests);
        assert_eq!(2, info.stats.requests_executed);
        assert_eq!(0, info.stats.cache_hits.all());
        assert_eq!(2, info.stats.cache_misses.all());
    });
}

#[test]
#[cfg_attr(not(feature = "dist-tests"), ignore)]
fn test_dist_nobuilder() {
    let tmpdir = tempfile::Builder::new()
        .prefix("ccache_dist_test")
        .tempdir()
        .unwrap();
    let tmpdir = tmpdir.path();
    let ccache_dist = harness::ccache_dist_path();

    let mut system = harness::DistSystem::new(&ccache_dist, tmpdir);
    system.add_scheduler();

    let ccache_cfg = dist_test_ccache_client_cfg(tmpdir, system.scheduler_url());
    let ccache_cfg_path = tmpdir.join("ccache-cfg.json");
    write_json_cfg(tmpdir, "ccache-cfg.json", &ccache_cfg);
    let ccache_cached_cfg_path = tmpdir.join("ccache-cached-cfg");

    stop_local_daemon();
    start_local_daemon(&ccache_cfg_path, &ccache_cached_cfg_path);
    basic_compile(tmpdir, &ccache_cfg_path, &ccache_cached_cfg_path);

    get_stats(|info| {
        assert_eq!(0, info.stats.dist_compiles.values().sum::<usize>());
        assert_eq!(1, info.stats.dist_errors);
        assert_eq!(1, info.stats.compile_requests);
        assert_eq!(1, info.stats.requests_executed);
        assert_eq!(0, info.stats.cache_hits.all());
        assert_eq!(1, info.stats.cache_misses.all());
    });
}

struct FailingServer;
impl ServerIncoming for FailingServer {
    fn handle_assign_job(&self, _job_id: JobId, _tc: Toolchain) -> Result<AssignJobResult> {
        let need_toolchain = false;
        let state = JobState::Ready;
        Ok(AssignJobResult {
            need_toolchain,
            state,
        })
    }
    fn handle_submit_toolchain(
        &self,
        _requester: &dyn ServerOutgoing,
        _job_id: JobId,
        _tc_rdr: ToolchainReader,
    ) -> Result<SubmitToolchainResult> {
        panic!("should not have submitted toolchain")
    }
    fn handle_run_job(
        &self,
        requester: &dyn ServerOutgoing,
        job_id: JobId,
        _command: CompileCommand,
        _outputs: Vec<String>,
        _inputs_rdr: InputsReader,
    ) -> Result<RunJobResult> {
        requester
            .do_update_job_state(job_id, JobState::Started)
            .context("Updating job state failed")?;
        bail!("internal build failure")
    }
}

#[test]
#[cfg_attr(not(feature = "dist-tests"), ignore)]
fn test_dist_failingserver() {
    let tmpdir = tempfile::Builder::new()
        .prefix("ccache_dist_test")
        .tempdir()
        .unwrap();
    let tmpdir = tmpdir.path();
    let ccache_dist = harness::ccache_dist_path();

    let mut system = harness::DistSystem::new(&ccache_dist, tmpdir);
    system.add_scheduler();
    system.add_custom_server(FailingServer);

    let ccache_cfg = dist_test_ccache_client_cfg(tmpdir, system.scheduler_url());
    let ccache_cfg_path = tmpdir.join("ccache-cfg.json");
    write_json_cfg(tmpdir, "ccache-cfg.json", &ccache_cfg);
    let ccache_cached_cfg_path = tmpdir.join("ccache-cached-cfg");

    stop_local_daemon();
    start_local_daemon(&ccache_cfg_path, &ccache_cached_cfg_path);
    basic_compile(tmpdir, &ccache_cfg_path, &ccache_cached_cfg_path);

    get_stats(|info| {
        assert_eq!(0, info.stats.dist_compiles.values().sum::<usize>());
        assert_eq!(1, info.stats.dist_errors);
        assert_eq!(1, info.stats.compile_requests);
        assert_eq!(1, info.stats.requests_executed);
        assert_eq!(0, info.stats.cache_hits.all());
        assert_eq!(1, info.stats.cache_misses.all());
    });
}
