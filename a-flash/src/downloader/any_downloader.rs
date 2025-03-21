use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{self, File};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Semaphore;
use url::Url;
use crate::downloader::error::DownloaderError;

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
    pub async fn download_from_file(&self, file_path: &str) -> Result<(), DownloaderError> {
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
            ProgressStyle::default_bar()
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
                        Ok(path) => pb.set_message(format!("Downloaded: {} -> {}", url, path.display())),
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
    base_output_dir: &str,
) -> Result<PathBuf, DownloaderError> {
    let parsed_url = Url::parse(url).map_err(|e| DownloaderError::UrlParse(e))?;

    // 提取域名后的路径
    let path_segments: Vec<&str> = parsed_url
        .path_segments()
        .map(|segments| segments.collect())
        .unwrap_or_default();

    if path_segments.is_empty() {
        return Err(DownloaderError::Other(format!("Invalid URL path: {}", url)));
    }

    // 最后一个段落是文件名
    let file_name = path_segments.last().unwrap();

    // 前面的段落是目录结构
    let dirs = &path_segments[..path_segments.len() - 1];

    // 创建完整的输出路径
    let mut output_path = PathBuf::from(base_output_dir);

    // 添加域名作为顶级目录
    let host = parsed_url.host_str().unwrap_or("unknown_host");
    output_path.push(host);

    // 添加URL中的路径
    for dir in dirs {
        if !dir.is_empty() {
            output_path.push(dir);
        }
    }

    // 确保目录存在
    fs::create_dir_all(&output_path).await?;

    // 添加文件名
    output_path.push(if file_name.is_empty() {
        format!("image_{}.jpg", uuid::Uuid::new_v4().simple().to_string())
    } else {
        file_name.to_string()
    });

    // 下载图片
    let response = client.get(url).send().await?;

    // 检查状态码
    if !response.status().is_success() {
        return Err(DownloaderError::Other(format!(
            "HTTP error status: {}",
            response.status()
        )));
    }

    let bytes = response.bytes().await?;

    // 保存文件
    fs::write(&output_path, &bytes).await?;

    Ok(output_path)
}

