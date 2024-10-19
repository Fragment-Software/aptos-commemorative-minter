use std::path::Path;

use aptos_sdk::types::LocalAccount;
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use term_size::dimensions;
use tokio::io::AsyncBufReadExt;

use crate::constants::PRIVATE_KEYS_FILE_PATH;

async fn read_file_lines(path: impl AsRef<Path>) -> eyre::Result<Vec<String>> {
    let file = tokio::fs::read(path).await?;
    let mut lines = file.lines();

    let mut contents = vec![];
    while let Some(line) = lines.next_line().await? {
        contents.push(line);
    }

    Ok(contents)
}

pub async fn read_private_keys() -> Vec<LocalAccount> {
    read_file_lines(PRIVATE_KEYS_FILE_PATH)
        .await
        .expect("Private keys file to be valid")
        .iter()
        .map(|pk| LocalAccount::from_private_key(pk, 0).expect("Private key to be valid"))
        .collect()
}

pub fn random_in_range<T>(range: [T; 2]) -> T
where
    T: rand::distributions::uniform::SampleUniform + PartialOrd + Copy,
{
    let start = range[0];
    let end = range[1];

    let inclusive_range = if start <= end {
        start..=end
    } else {
        end..=start
    };

    rand::thread_rng().gen_range(inclusive_range)
}

pub async fn pretty_sleep(sleep_range: [u64; 2]) {
    let random_sleep_duration_secs = random_in_range(sleep_range);

    let pb = ProgressBar::new(random_sleep_duration_secs);

    let term_width = dimensions().map(|(w, _)| w - 2).unwrap_or(40);
    let bar_width = if term_width > 20 { term_width - 20 } else { 20 };

    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{spinner:.green}} [{{elapsed_precise}}] [{{bar:{bar_width}.cyan/blue}}] {{pos}}/{{len}}s"
            ))
            .expect("Invalid progress bar template.")
            .progress_chars("#>-"),
    );

    let step = std::time::Duration::from_secs(1);

    for _ in 0..random_sleep_duration_secs {
        pb.inc(1);
        tokio::time::sleep(step).await;
    }

    pb.finish_with_message("Done!");
}
