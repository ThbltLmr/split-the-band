use procfs::process::all_processes;
use std::{
    fs::File,
    io::{self, BufRead},
    thread,
    time::Duration,
};
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};

fn read_system_bandwidth() -> io::Result<(u64, u64)> {
    let file = File::open("/proc/net/dev")?;
    let reader = io::BufReader::new(file);
    let mut total_received = 0;
    let mut total_transmitted = 0;

    for line in reader.lines().skip(2) {
        // Skip headers
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        total_received += parts[1].parse::<u64>().unwrap_or(0);
        total_transmitted += parts[9].parse::<u64>().unwrap_or(0);
    }
    Ok((total_received, total_transmitted))
}

fn monitor_bandwidth(interval: Duration) -> io::Result<()> {
    let mut previous = read_system_bandwidth()?;
    println!(
        "Monitoring bandwidth usage (refresh every {} seconds):",
        interval.as_secs()
    );
    println!("Press Ctrl+C to stop.\n");

    loop {
        thread::sleep(interval);

        let current = read_system_bandwidth()?;
        let received_per_sec = current.0.saturating_sub(previous.0);
        let transmitted_per_sec = current.1.saturating_sub(previous.1);

        println!(
            "Download: {:.2} KB/sec, Upload: {:.2} KB/sec",
            received_per_sec as f64 / 1024.0,
            transmitted_per_sec as f64 / 1024.0
        );

        previous = current;
    }
}

fn list_processes() -> Result<(), Box<dyn std::error::Error>> {
    let system = System::new_all();

    // Get all processes
    for process in all_processes()? {
        let pid = process?.pid();
        if let Some(sys_process) = system.process(Pid::from_u32(pid.try_into().unwrap())) {
            let name = sys_process.name();
            println!("PID: {}, Name: {}", pid, name);
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = monitor_bandwidth(Duration::from_secs(2)) {
        eprintln!("Error reading system bandwidth: {}", e);
    }
}
