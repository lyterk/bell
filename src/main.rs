use chrono::{Local, Timelike};
use log::{error, info};
use rand::RngExt;
use rand_distr::{Distribution, Exp};
use rodio::{Decoder, MixerDeviceSink, Player};
use std::fs;
use std::io::Write;
use std::thread;
use std::time::Duration;

fn is_quiet_hours() -> bool {
    let hour = Local::now().hour();
    hour >= 23 || hour < 7
}

fn play_random_file() -> Result<(), Box<dyn std::error::Error>> {
    let entries: Vec<_> = fs::read_dir(".")?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    if entries.is_empty() {
        error!("No files found in resources/");
        return Ok(());
    }

    let idx = rand::rng().random_range(0..entries.len());
    let path = entries[idx].path();

    let mut handle: MixerDeviceSink = rodio::DeviceSinkBuilder::open_default_sink()?;
    handle.log_on_drop(false);
    let player = Player::connect_new(&handle.mixer());

    let file = fs::File::open(&path)?;
    let source = Decoder::try_from(std::io::BufReader::new(file))?;
    player.append(source);
    player.sleep_until_end();

    info!("Played: {:?}", path);
    Ok(())
}

fn main() {
    env_logger::builder()
        .format(|buf, record| {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            writeln!(
                buf,
                "[{}] {} - {}",
                timestamp,
                record.level(),
                record.args()
            )
        })
        .filter_level(log::LevelFilter::Info)
        .init();

    let exp = Exp::new(1.0f64).expect("Failed to create exponential distribution");
    let thirty_minutes_secs = 1.0 * 60.0;

    loop {
        let sample = exp.sample(&mut rand::rng());
        let wait_secs = (sample * thirty_minutes_secs) as u64;

        info!(
            "Next bell in {} seconds ({:.1} minutes)",
            wait_secs,
            wait_secs as f64 / 60.0
        );
        thread::sleep(Duration::from_secs(wait_secs));

        if is_quiet_hours() {
            info!("Quiet hours (11pm–7am), skipping bell.");
        } else {
            if let Err(e) = play_random_file() {
                error!("Error playing file: {}", e);
            }
        }
    }
}
