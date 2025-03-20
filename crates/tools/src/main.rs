use futures::stream::{self, StreamExt};
use indicatif::ProgressBar;
use reqwest::Client;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::{self, File};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Semaphore;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Other error: {0}")]
    Other(String),
}

pub struct ImageDownloader {
    client: Client,
    output_dir: String,
    max_concurrent: usize,
}

impl ImageDownloader {
    pub fn new(output_dir: &str, max_concurrent: usize) -> Self {
        Self {
            client: Client::new(),
            output_dir: output_dir.to_string(),
            max_concurrent,
        }
    }

    // 从文件读取URLs并下载图片
    pub async fn download_from_file(&self, file_path: &str) -> Result<(), DownloadError> {
        // 确保输出目录存在
        fs::create_dir_all(&self.output_dir).await?;

        // 读取文件
        let file = File::open(file_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // 收集所有URLs
        let mut urls = Vec::new();
        while let Some(line) = lines.next_line().await? {
            let line = line.trim();
            if !line.is_empty() {
                urls.push(line.to_string());
            }
        }

        // 创建进度条
        let progress_bar = Arc::new(ProgressBar::new(urls.len() as u64));
        progress_bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );

        // 创建信号量控制并发
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));

        // 并发下载
        let results = stream::iter(urls)
            .map(|url| {
                let client = self.client.clone();
                let output_dir = self.output_dir.clone();
                let sem = Arc::clone(&semaphore);
                let pb = Arc::clone(&progress_bar);

                async move {
                    let _permit = sem.acquire().await;
                    let result = download_image(&client, &url, &output_dir).await;
                    pb.inc(1);

                    match &result {
                        Ok(_) => pb.set_message(format!("Downloaded: {}", url)),
                        Err(e) => pb.set_message(format!("Failed: {} - {}", url, e)),
                    }

                    result
                }
            })
            .buffer_unordered(self.max_concurrent)
            .collect::<Vec<_>>()
            .await;

        // 处理结果
        let mut success_count = 0;
        let mut failure_count = 0;

        for result in results {
            match result {
                Ok(_) => success_count += 1,
                Err(e) => {
                    failure_count += 1;
                    eprintln!("Download failed: {}", e);
                }
            }
        }

        progress_bar.finish_with_message(format!(
            "Completed! Success: {}, Failed: {}",
            success_count, failure_count
        ));

        Ok(())
    }
}

async fn download_image(
    client: &Client,
    url: &str,
    output_dir: &str,
) -> Result<(), DownloadError> {
    // 从URL中提取文件名
    let file_name = Url::parse(url)
        .ok()
        .and_then(|url| url.path_segments()?.last().map(String::from))
        .unwrap_or_else(|| {
            format!(
                "image_{}.jpg",
                uuid::Uuid::new_v4().simple().to_string()
            )
        });

    // 构建输出路径
    let output_path = Path::new(output_dir).join(&file_name);

    // 下载图片
    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;

    // 保存文件
    fs::write(&output_path, &bytes).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), DownloadError> {
    let downloader = ImageDownloader::new("downloaded_images", 5);
    downloader.download_from_file("D:\\RustroverProjects\\sakura\\crates\\tools\\src\\image_urls.text").await?;
    Ok(())
}