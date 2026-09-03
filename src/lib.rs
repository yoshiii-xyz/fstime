//! Explainable, bounded filesystem traversal measurements.
//!
//! The profiler counts only filesystem operations it invokes. It does not
//! claim to observe kernel scheduling, cache state, or every operation made by
//! the operating system. Symlinks are inspected but never followed.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;
use std::time::Instant;

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_ENTRIES: usize = 100_000;
pub const MAX_DIRECTORY_METRICS: usize = 10_000;
pub const MAX_ERRORS: usize = 32;
pub const MAX_DEPTH: usize = 256;
pub const MAX_REPORT_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationCounts {
    pub traversal_count: u64,
    pub open_count: u64,
    pub stat_count: u64,
    pub readdir_count: u64,
    pub readlink_count: u64,
    pub realpath_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryMetric {
    pub path: String,
    pub direct_entry_count: u64,
    pub subtree_file_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileReport {
    pub schema_version: u32,
    pub root: String,
    pub cache_label: String,
    pub elapsed_ms: u64,
    pub complete: bool,
    pub operations: OperationCounts,
    pub visited_count: u64,
    pub ignored_count: u64,
    pub file_count: u64,
    pub directory_count: u64,
    pub symlink_count: u64,
    pub other_count: u64,
    pub permission_failures: u64,
    pub directories: Vec<DirectoryMetric>,
    pub errors: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricDelta {
    pub metric: String,
    pub left: u64,
    pub right: u64,
    pub right_minus_left: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileComparison {
    pub schema_version: u32,
    pub left_cache_label: String,
    pub right_cache_label: String,
    pub deltas: Vec<MetricDelta>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileOptions {
    pub ignore: Vec<String>,
    pub cache_label: String,
}

impl Default for ProfileOptions {
    fn default() -> Self {
        Self {
            ignore: Vec::new(),
            cache_label: "unspecified".to_owned(),
        }
    }
}

/// Profile one local tree without following symlinks.
pub fn profile_tree(root: &Path, options: &ProfileOptions) -> ProfileReport {
    let started = Instant::now();
    let mut state = ProfileState::new(options);
    state.stat_count += 1;
    match fs::symlink_metadata(root) {
        Ok(metadata) if metadata.is_dir() => {
            state.directory_count += 1;
            walk_directory(root, Path::new(""), 0, &mut state);
        }
        Ok(_) => push_error(
            &mut state.errors,
            format!("root {} is not a directory", display_path(root)),
        ),
        Err(error) => {
            record_error(&mut state, &format!("root {}", display_path(root)), error);
        }
    }
    state
        .directories
        .sort_by(|left, right| left.path.cmp(&right.path));
    let elapsed_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let mut notes = vec![
        "elapsed_ms is a volatile wall-clock measurement".to_owned(),
        "open_count and realpath_count remain zero unless this implementation invokes those operations".to_owned(),
        "cache_label records the operator-selected label and does not prove cache state".to_owned(),
    ];
    if state.directories.len() == MAX_DIRECTORY_METRICS {
        notes.push(format!(
            "directory metrics are capped at {} records",
            MAX_DIRECTORY_METRICS
        ));
    }
    ProfileReport {
        schema_version: SCHEMA_VERSION,
        root: display_path(root),
        cache_label: options.cache_label.clone(),
        elapsed_ms,
        complete: state.errors.is_empty(),
        operations: OperationCounts {
            traversal_count: state.traversal_count,
            open_count: 0,
            stat_count: state.stat_count,
            readdir_count: state.readdir_count,
            readlink_count: state.readlink_count,
            realpath_count: 0,
        },
        visited_count: state.visited_count,
        ignored_count: state.ignored_count,
        file_count: state.file_count,
        directory_count: state.directory_count,
        symlink_count: state.symlink_count,
        other_count: state.other_count,
        permission_failures: state.permission_failures,
        directories: state.directories,
        errors: state.errors,
        notes: std::mem::take(&mut notes),
    }
}

/// Compare numeric measurements from two profile reports.
pub fn compare_profiles(left: &ProfileReport, right: &ProfileReport) -> ProfileComparison {
    let left_metrics = metric_values(left);
    let right_metrics = metric_values(right);
    let deltas = left_metrics
        .iter()
        .zip(right_metrics.iter())
        .map(|((metric, left_value), (_, right_value))| MetricDelta {
            metric: (*metric).to_owned(),
            left: *left_value,
            right: *right_value,
            right_minus_left: signed_delta(*left_value, *right_value),
        })
        .collect();
    ProfileComparison {
        schema_version: SCHEMA_VERSION,
        left_cache_label: left.cache_label.clone(),
        right_cache_label: right.cache_label.clone(),
        deltas,
        notes: vec![
            "deltas describe observed userspace measurements, not causal kernel timings".to_owned(),
            "elapsed_ms is volatile and should be compared across controlled runs".to_owned(),
        ],
    }
}

/// Serialize a profile while enforcing the JSON report size bound.
pub fn profile_json(report: &ProfileReport) -> Result<String, String> {
    bounded_json(report)
}

/// Serialize a comparison while enforcing the JSON report size bound.
pub fn comparison_json(report: &ProfileComparison) -> Result<String, String> {
    bounded_json(report)
}

/// Explain a profile report without rereading the filesystem.
pub fn explain_profile(report: &ProfileReport) -> String {
    let mut lines = vec![
        format!("root: {}", report.root),
        format!("cache label: {}", report.cache_label),
        format!("complete: {}", report.complete),
        format!("elapsed_ms: {}", report.elapsed_ms),
        format!("visited: {}", report.visited_count),
        format!("ignored: {}", report.ignored_count),
        format!("files: {}", report.file_count),
        format!("directories: {}", report.directory_count),
        format!("symlinks: {}", report.symlink_count),
        format!("permission failures: {}", report.permission_failures),
        format!("stat calls: {}", report.operations.stat_count),
        format!("readdir calls: {}", report.operations.readdir_count),
        format!("readlink calls: {}", report.operations.readlink_count),
        "differences not applicable: profile has no comparison result".to_owned(),
    ];
    if !report.errors.is_empty() {
        lines.push("errors:".to_owned());
        lines.extend(report.errors.iter().map(|error| format!("- {error}")));
    }
    lines.join("\n")
}

/// Explain a profile comparison without rereading either input tree.
pub fn explain_comparison(report: &ProfileComparison) -> String {
    let mut lines = vec![
        format!("left cache label: {}", report.left_cache_label),
        format!("right cache label: {}", report.right_cache_label),
        "deltas:".to_owned(),
    ];
    lines.extend(report.deltas.iter().map(|delta| {
        format!(
            "- {}: left={}, right={}, delta={}",
            delta.metric, delta.left, delta.right, delta.right_minus_left
        )
    }));
    lines
        .into_iter()
        .chain(report.notes.iter().map(|note| format!("note: {note}")))
        .collect::<Vec<_>>()
        .join("\n")
}

struct ProfileState<'a> {
    options: &'a ProfileOptions,
    traversal_count: u64,
    stat_count: u64,
    readdir_count: u64,
    readlink_count: u64,
    visited_count: u64,
    ignored_count: u64,
    file_count: u64,
    directory_count: u64,
    symlink_count: u64,
    other_count: u64,
    permission_failures: u64,
    directories: Vec<DirectoryMetric>,
    errors: Vec<String>,
    entry_count: usize,
}

impl<'a> ProfileState<'a> {
    fn new(options: &'a ProfileOptions) -> Self {
        Self {
            options,
            traversal_count: 0,
            stat_count: 0,
            readdir_count: 0,
            readlink_count: 0,
            visited_count: 0,
            ignored_count: 0,
            file_count: 0,
            directory_count: 0,
            symlink_count: 0,
            other_count: 0,
            permission_failures: 0,
            directories: Vec::new(),
            errors: Vec::new(),
            entry_count: 0,
        }
    }
}

fn walk_directory(root: &Path, relative: &Path, depth: usize, state: &mut ProfileState<'_>) -> u64 {
    if depth > MAX_DEPTH {
        push_error(
            &mut state.errors,
            format!(
                "depth exceeds {} at {}",
                MAX_DEPTH,
                relative_display(relative)
            ),
        );
        return 0;
    }
    state.readdir_count += 1;
    let iterator = match fs::read_dir(root.join(relative)) {
        Ok(iterator) => iterator,
        Err(error) => {
            record_error(
                state,
                &format!("read_dir {}", relative_display(relative)),
                error,
            );
            return 0;
        }
    };
    let mut children = Vec::new();
    for item in iterator {
        match item {
            Ok(entry) => children.push(entry),
            Err(error) => record_error(state, "read_dir entry", error),
        }
    }
    children.sort_by_key(|entry| entry.file_name());
    let mut direct_entry_count = 0u64;
    let mut subtree_file_bytes = 0u64;
    for child in children {
        let child_relative = relative.join(child.file_name());
        if should_ignore(&child_relative, &state.options.ignore) {
            state.ignored_count += 1;
            continue;
        }
        if state.entry_count >= MAX_ENTRIES {
            push_error(
                &mut state.errors,
                format!("tree exceeds {} entries", MAX_ENTRIES),
            );
            break;
        }
        state.entry_count += 1;
        state.traversal_count += 1;
        state.visited_count += 1;
        direct_entry_count += 1;
        state.stat_count += 1;
        let absolute = root.join(&child_relative);
        let metadata = match fs::symlink_metadata(&absolute) {
            Ok(metadata) => metadata,
            Err(error) => {
                record_error(
                    state,
                    &format!("metadata {}", relative_display(&child_relative)),
                    error,
                );
                continue;
            }
        };
        if metadata.is_file() {
            state.file_count += 1;
            subtree_file_bytes = subtree_file_bytes.saturating_add(metadata.len());
        } else if metadata.is_dir() {
            state.directory_count += 1;
            subtree_file_bytes = subtree_file_bytes.saturating_add(walk_directory(
                root,
                &child_relative,
                depth + 1,
                state,
            ));
        } else if metadata.file_type().is_symlink() {
            state.symlink_count += 1;
            state.readlink_count += 1;
            if let Err(error) = fs::read_link(&absolute) {
                record_error(
                    state,
                    &format!("readlink {}", relative_display(&child_relative)),
                    error,
                );
            }
        } else {
            state.other_count += 1;
        }
    }
    if state.directories.len() < MAX_DIRECTORY_METRICS {
        state.directories.push(DirectoryMetric {
            path: relative_display(relative),
            direct_entry_count,
            subtree_file_bytes,
        });
    }
    subtree_file_bytes
}

fn metric_values(report: &ProfileReport) -> Vec<(&'static str, u64)> {
    vec![
        ("elapsed_ms", report.elapsed_ms),
        ("traversal_count", report.operations.traversal_count),
        ("open_count", report.operations.open_count),
        ("stat_count", report.operations.stat_count),
        ("readdir_count", report.operations.readdir_count),
        ("readlink_count", report.operations.readlink_count),
        ("realpath_count", report.operations.realpath_count),
        ("visited_count", report.visited_count),
        ("ignored_count", report.ignored_count),
        ("file_count", report.file_count),
        ("directory_count", report.directory_count),
        ("symlink_count", report.symlink_count),
        ("other_count", report.other_count),
        ("permission_failures", report.permission_failures),
    ]
}

fn signed_delta(left: u64, right: u64) -> i64 {
    let delta = i128::from(right) - i128::from(left);
    delta.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

fn bounded_json<T: Serialize>(value: &T) -> Result<String, String> {
    let rendered = serde_json::to_string(value).map_err(|error| error.to_string())?;
    if rendered.len() > MAX_REPORT_BYTES {
        return Err(format!(
            "report is {} bytes, maximum is {} bytes",
            rendered.len(),
            MAX_REPORT_BYTES
        ));
    }
    Ok(rendered)
}

fn should_ignore(relative: &Path, patterns: &[String]) -> bool {
    let display = relative_display(relative);
    patterns.iter().any(|pattern| {
        let pattern = pattern.trim_matches('/');
        !pattern.is_empty()
            && (display == pattern
                || relative
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy() == pattern)
                || display.starts_with(&format!("{pattern}/")))
    })
}

fn record_error(state: &mut ProfileState<'_>, operation: &str, error: io::Error) {
    if error.kind() == io::ErrorKind::PermissionDenied {
        state.permission_failures += 1;
    }
    push_error(&mut state.errors, format!("{operation}: {error}"));
}

fn push_error(errors: &mut Vec<String>, error: String) {
    if errors.len() < MAX_ERRORS {
        errors.push(error);
    }
}

fn display_path(path: &Path) -> String {
    let rendered = path.to_string_lossy().replace('\\', "/");
    if rendered.is_empty() {
        ".".to_owned()
    } else {
        rendered
    }
}

fn relative_display(path: &Path) -> String {
    display_path(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::path::PathBuf;

    #[cfg(unix)]
    use std::os::unix::fs::symlink;

    fn temp_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("fstime-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn options(ignore: &[&str]) -> ProfileOptions {
        ProfileOptions {
            ignore: ignore.iter().map(|item| (*item).to_owned()).collect(),
            cache_label: "cold".to_owned(),
        }
    }

    #[test]
    fn small_tree_counts_operations_and_sizes() {
        let root = temp_root("small");
        fs::create_dir(root.join("nested")).unwrap();
        fs::write(root.join("nested/data"), b"four").unwrap();
        let report = profile_tree(&root, &options(&[]));
        assert!(report.complete);
        assert_eq!(report.file_count, 1);
        assert_eq!(report.directory_count, 2);
        assert!(report.operations.stat_count >= 3);
        assert_eq!(report.operations.readdir_count, 2);
        assert_eq!(report.directories[0].path, ".");
        assert_eq!(report.directories[0].subtree_file_bytes, 4);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn deep_tree_hits_bound() {
        let root = temp_root("deep");
        let mut current = root.clone();
        for index in 0..(MAX_DEPTH + 2) {
            current = current.join(format!("d{index}"));
            fs::create_dir(&current).unwrap();
        }
        let report = profile_tree(&root, &options(&[]));
        assert!(!report.complete);
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("depth exceeds"))
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ignored_files_are_not_visited() {
        let root = temp_root("ignored");
        fs::write(root.join("keep"), b"x").unwrap();
        fs::write(root.join("ignored"), b"x").unwrap();
        let report = profile_tree(&root, &options(&["ignored"]));
        assert_eq!(report.ignored_count, 1);
        assert_eq!(report.file_count, 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn symlink_loop_is_inspected_without_following() {
        let root = temp_root("loop");
        symlink("b", root.join("a")).unwrap();
        symlink("a", root.join("b")).unwrap();
        let report = profile_tree(&root, &options(&[]));
        assert!(report.complete);
        assert_eq!(report.symlink_count, 2);
        assert_eq!(report.operations.readlink_count, 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_root_is_inconclusive() {
        let root = temp_root("missing");
        let report = profile_tree(&root.join("missing"), &options(&[]));
        assert!(!report.complete);
        assert_eq!(report.errors.len(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unicode_name_is_retained() {
        let root = temp_root("unicode");
        fs::write(root.join("spä ce"), b"x").unwrap();
        let report = profile_tree(&root, &options(&[]));
        assert!(report.complete);
        assert_eq!(report.file_count, 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn repeated_profile_has_stable_non_timing_fields() {
        let root = temp_root("stable");
        fs::write(root.join("data"), b"fixed").unwrap();
        let mut first = profile_tree(&root, &options(&[]));
        let mut second = profile_tree(&root, &options(&[]));
        first.elapsed_ms = 0;
        second.elapsed_ms = 0;
        assert_eq!(first, second);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn report_size_bound_is_enforced() {
        let root = temp_root("bound");
        let mut report = profile_tree(&root, &options(&[]));
        report.notes.push("x".repeat(MAX_REPORT_BYTES));
        assert!(profile_json(&report).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn compare_includes_observed_metrics() {
        let root = temp_root("compare");
        File::create(root.join("data")).unwrap();
        let report = profile_tree(&root, &options(&[]));
        let comparison = compare_profiles(&report, &report);
        assert_eq!(comparison.deltas.len(), 14);
        assert!(
            comparison
                .deltas
                .iter()
                .all(|delta| delta.right_minus_left == 0)
        );
        fs::remove_dir_all(root).unwrap();
    }
}
