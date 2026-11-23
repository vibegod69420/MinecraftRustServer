use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

pub struct SummaryReporter {
    on_deregistration: Box<dyn Fn()>,
}

impl SummaryReporter {
    // JAVA: net.minecraft.util.profiling.jfr.SummaryReporter#<init>
    pub fn new(on_deregistration: Box<dyn Fn()>) -> Self {
        Self {
            on_deregistration,
        }
    }

    // JAVA: net.minecraft.util.profiling.jfr.SummaryReporter#recordingStopped
    //
    // The `parse_jfr` parameter is a self-contained adaptation replacing
    // `JfrStatsParser.parse(path)` + `JfrStatsResult.asJson()`.
    pub fn recording_stopped<F, E>(&self, path: Option<&Path>, parse_jfr: F)
    where
        F: FnOnce(&Path) -> Result<String, E>,
        E: std::fmt::Display,
    {
        let Some(path) = path else {
            return;
        };

        (self.on_deregistration)();
        info_with_fallback(&|| {
            format!("Dumped flight recorder profiling to {}", path.display())
        });

        let json = match catch_unwind(AssertUnwindSafe(|| parse_jfr(path))) {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                warn_with_fallback(&|| "Failed to parse JFR recording".to_string(), &e);
                return;
            }
            Err(_) => {
                warn_with_fallback(
                    &|| "Failed to parse JFR recording".to_string(),
                    &"panic during JFR parse",
                );
                return;
            }
        };

        let write_result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
            info_with_fallback(&|| json.clone());

            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("recording.jfr");
            let base = file_name.strip_suffix(".jfr").unwrap_or(file_name);
            let report_name = format!("jfr-report-{}.json", base);
            let report_path = path.with_file_name(report_name);

            fs::write(&report_path, &json).map_err(|e| e.to_string())?;
            info_with_fallback(&|| {
                format!("Dumped recording summary to {}", report_path.display())
            });
            Ok(())
        }));

        match write_result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                warn_with_fallback(&|| "Failed to output JFR report".to_string(), &e);
            }
            Err(_) => {
                warn_with_fallback(
                    &|| "Failed to output JFR report".to_string(),
                    &"panic during JFR report output",
                );
            }
        }
    }
}

// JAVA: net.minecraft.util.profiling.jfr.SummaryReporter#infoWithFallback
fn info_with_fallback(msg: &dyn Fn() -> String) {
    println!("{}", msg());
}

// JAVA: net.minecraft.util.profiling.jfr.SummaryReporter#warnWithFallback
fn warn_with_fallback(msg: &dyn Fn() -> String, err: &dyn std::fmt::Display) {
    eprintln!("{}", msg());
    eprintln!("{}", err);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("mc_math_summary_{}", name));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn none_path_does_not_call_deregistration() {
        let called = Arc::new(AtomicBool::new(false));
        let c2 = called.clone();
        let reporter = SummaryReporter::new(Box::new(move || {
            c2.store(true, Ordering::SeqCst);
        }));
        reporter.recording_stopped(None, |_p: &Path| Ok::<String, String>("{}".to_string()));
        assert!(!called.load(Ordering::SeqCst));
    }

    #[test]
    fn successful_parse_writes_report() {
        let dir = temp_dir("success");
        let jfr_path = dir.join("profile.jfr");
        fs::write(&jfr_path, b"dummy").unwrap();

        let reporter = SummaryReporter::new(Box::new(|| {}));
        reporter.recording_stopped(Some(&jfr_path), |_p: &Path| {
            Ok::<String, String>("{\"ok\":true}".to_string())
        });

        let report = dir.join("jfr-report-profile.json");
        assert_eq!(fs::read_to_string(&report).unwrap(), "{\"ok\":true}");

        let _ = fs::remove_file(&jfr_path);
        let _ = fs::remove_file(&report);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn parse_error_does_not_write_report() {
        let dir = temp_dir("parse_err");
        let jfr_path = dir.join("bad.jfr");
        fs::write(&jfr_path, b"dummy").unwrap();

        let reporter = SummaryReporter::new(Box::new(|| {}));
        reporter.recording_stopped(Some(&jfr_path), |_p: &Path| {
            Err::<String, String>("parse fail".to_string())
        });

        let report = dir.join("jfr-report-bad.json");
        assert!(!report.exists());

        let _ = fs::remove_file(&jfr_path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn calls_deregistration_before_parse() {
        let dir = temp_dir("dereg");
        let jfr_path = dir.join("test.jfr");
        fs::write(&jfr_path, b"dummy").unwrap();

        let called = Arc::new(AtomicBool::new(false));
        let c2 = called.clone();
        let reporter = SummaryReporter::new(Box::new(move || {
            c2.store(true, Ordering::SeqCst);
        }));

        reporter.recording_stopped(Some(&jfr_path), |_p: &Path| {
            Ok::<String, String>("{}".to_string())
        });

        assert!(called.load(Ordering::SeqCst));

        let _ = fs::remove_file(&jfr_path);
        let _ = fs::remove_file(dir.join("jfr-report-test.json"));
        let _ = fs::remove_dir(&dir);
    }
}
