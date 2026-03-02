use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct ProcessSnapshot {
    pub pid: u32,
    pub comm: String,
    pub rss_bytes: u64,
    pub vss_bytes: u64,
    pub minor_faults: u64,
    pub major_faults: u64,
}

#[derive(Debug, Clone, Default)]
pub struct SystemMemorySnapshot {
    pub available_bytes: u64,
    pub cache_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct ProcSnapshot {
    pub processes: Vec<ProcessSnapshot>,
    pub system_memory: SystemMemorySnapshot,
}

#[derive(Debug, Clone)]
pub struct ProcReader {
    proc_root: PathBuf,
    page_size: u64,
}

impl ProcReader {
    pub fn new(proc_root: impl AsRef<Path>) -> Self {
        Self {
            proc_root: proc_root.as_ref().to_path_buf(),
            page_size: page_size_bytes(),
        }
    }

    pub fn collect(&self) -> Result<ProcSnapshot> {
        let mut processes = Vec::new();

        for entry in fs::read_dir(&self.proc_root)
            .with_context(|| format!("failed to list {}", self.proc_root.display()))?
        {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            let Ok(pid) = file_name.parse::<u32>() else {
                continue;
            };

            if let Some(snapshot) = self.read_process(pid) {
                processes.push(snapshot);
            }
        }

        let system_memory = self.read_meminfo()?;

        Ok(ProcSnapshot {
            processes,
            system_memory,
        })
    }

    fn read_process(&self, pid: u32) -> Option<ProcessSnapshot> {
        let process_dir = self.proc_root.join(pid.to_string());

        let stat = fs::read_to_string(process_dir.join("stat")).ok()?;
        let statm = fs::read_to_string(process_dir.join("statm")).ok()?;

        let parsed_stat = parse_stat(&stat)?;
        let parsed_statm = parse_statm(&statm, self.page_size)?;

        Some(ProcessSnapshot {
            pid,
            comm: parsed_stat.comm,
            rss_bytes: parsed_statm.rss_bytes,
            vss_bytes: parsed_statm.vss_bytes,
            minor_faults: parsed_stat.minor_faults,
            major_faults: parsed_stat.major_faults,
        })
    }

    fn read_meminfo(&self) -> Result<SystemMemorySnapshot> {
        let meminfo_path = self.proc_root.join("meminfo");
        let meminfo = fs::read_to_string(&meminfo_path)
            .with_context(|| format!("failed to read {}", meminfo_path.display()))?;

        Ok(parse_meminfo(&meminfo))
    }
}

#[derive(Debug)]
struct ParsedStat {
    comm: String,
    minor_faults: u64,
    major_faults: u64,
}

#[derive(Debug)]
struct ParsedStatm {
    rss_bytes: u64,
    vss_bytes: u64,
}

fn parse_stat(raw: &str) -> Option<ParsedStat> {
    let open = raw.find('(')?;
    let close = raw.rfind(')')?;
    let comm = raw.get(open + 1..close)?.to_string();

    let rest = raw.get(close + 2..)?;
    let fields: Vec<&str> = rest.split_whitespace().collect();
    if fields.len() < 10 {
        return None;
    }

    let minor_faults = fields.get(7)?.parse().ok()?;
    let major_faults = fields.get(9)?.parse().ok()?;

    Some(ParsedStat {
        comm,
        minor_faults,
        major_faults,
    })
}

fn parse_statm(raw: &str, page_size: u64) -> Option<ParsedStatm> {
    let fields: Vec<&str> = raw.split_whitespace().collect();
    if fields.len() < 2 {
        return None;
    }

    let total_pages: u64 = fields.first()?.parse().ok()?;
    let rss_pages: u64 = fields.get(1)?.parse().ok()?;

    Some(ParsedStatm {
        rss_bytes: rss_pages.saturating_mul(page_size),
        vss_bytes: total_pages.saturating_mul(page_size),
    })
}

fn parse_meminfo(raw: &str) -> SystemMemorySnapshot {
    let mut snapshot = SystemMemorySnapshot::default();

    for line in raw.lines() {
        if let Some((name, value)) = line.split_once(':') {
            let kb = value
                .split_whitespace()
                .next()
                .and_then(|field| field.parse::<u64>().ok())
                .unwrap_or_default();

            let bytes = kb.saturating_mul(1024);
            match name {
                "MemAvailable" => snapshot.available_bytes = bytes,
                "Cached" => snapshot.cache_bytes = bytes,
                "MemTotal" => snapshot.total_bytes = bytes,
                _ => {}
            }
        }
    }

    snapshot
}

fn page_size_bytes() -> u64 {
    #[allow(clippy::cast_sign_loss)]
    {
        let raw = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        if raw <= 0 { 4096 } else { raw as u64 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parse_stat_extracts_comm_and_faults() {
        let sample = "1234 (my-process) R 1 1 1 0 -1 4194304 10 0 3 0 100 200 0 0 20 0 1 0";
        let parsed = parse_stat(sample).expect("stat should parse");

        assert_eq!(parsed.comm, "my-process");
        assert_eq!(parsed.minor_faults, 10);
        assert_eq!(parsed.major_faults, 3);
    }

    #[test]
    fn parse_statm_extracts_memory_sizes() {
        let parsed = parse_statm("100 25 10 0 0 0 0", 4096).expect("statm should parse");
        assert_eq!(parsed.vss_bytes, 409_600);
        assert_eq!(parsed.rss_bytes, 102_400);
    }

    #[test]
    fn parse_meminfo_extracts_key_fields() {
        let meminfo = "MemTotal:       1024 kB\nMemAvailable:    512 kB\nCached:          64 kB\n";
        let parsed = parse_meminfo(meminfo);

        assert_eq!(parsed.total_bytes, 1_048_576);
        assert_eq!(parsed.available_bytes, 524_288);
        assert_eq!(parsed.cache_bytes, 65_536);
    }

    #[test]
    fn collect_reads_mock_proc_fixture() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/mock_proc");
        let reader = ProcReader::new(fixture);

        let snapshot = reader.collect().expect("fixture snapshot should parse");
        assert_eq!(snapshot.processes.len(), 2);
        assert_eq!(snapshot.system_memory.total_bytes, 2_097_152_000);
    }
}
