use std::collections::HashSet;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use scraper::{Html, Selector};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::sleep;
use std::time::Duration;
use url::Url;
use uuid::Uuid;

// 自定义错误类型
#[derive(Debug, thiserror::Error)]
pub enum DownloaderError {
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Invalid file type")]
    InvalidFileType,
    #[error("Other error: {0}")]
    Other(String),
}

// 下载器结构体
#[derive(Clone)]
struct ImageDownloader {
    client: Client,
    base_url: Url,
    visited_urls: Arc<Mutex<HashSet<String>>>,
    downloaded_images: Arc<Mutex<HashSet<String>>>,
    output_dir: String,
    download_semaphore: Arc<Semaphore>,
    progress_bar: Arc<ProgressBar>,
}

impl ImageDownloader {
    // 创建新的下载器实例
    fn new(base_url: &str, output_dir: &str, max_concurrent: usize) -> Result<Self, DownloaderError> {
        let progress_bar = ProgressBar::new(0);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );

        Ok(Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .build()?,
            base_url: Url::parse(base_url)?,
            visited_urls: Arc::new(Mutex::new(HashSet::new())),
            downloaded_images: Arc::new(Mutex::new(HashSet::new())),
            output_dir: output_dir.to_string(),
            download_semaphore: Arc::new(Semaphore::new(max_concurrent)),
            progress_bar: Arc::new(progress_bar),
        })
    }

    // 下载器入口方法
    fn download_images<'a>(
        &'a mut self,
        url: &'a str,
        max_concurrent_pages: usize,
    ) -> Pin<Box<dyn Future<Output = Result<(), DownloaderError>> + 'a>> {
        Box::pin(self.download_images_inner(url, max_concurrent_pages))
    }

    // 实际的下载逻辑
    async fn download_images_inner(
        &mut self,
        url: &str,
        max_concurrent_pages: usize,
    ) -> Result<(), DownloaderError> {
        // 检查是否已访问
        {
            let visited = self.visited_urls.lock().await;
            if visited.contains(url) {
                return Ok(());
            }
        }

        // 标记为已访问
        {
            let mut visited = self.visited_urls.lock().await;
            visited.insert(url.to_string());
        }

        println!("Visiting: {}", url);
        sleep(Duration::from_millis(500)).await;

        // 获取页面内容
        let response = self.client.get(url).send().await?;
        let html = response.text().await?;
        let document = Html::parse_document(&html);

        // 确保输出目录存在
        fs::create_dir_all(&self.output_dir).await?;

        // 处理图片
        self.process_images(&document).await?;

        // 处理链接
        let page_semaphore = Arc::new(Semaphore::new(max_concurrent_pages));
        let link_selector = Selector::parse("a").unwrap();
        let mut page_tasks = vec![];

        for link in document.select(&link_selector) {
            if let Some(href) = link.value().attr("href") {
                if let Ok(absolute_url) = self.resolve_url(href) {
                    if self.should_visit_url(&absolute_url) {
                        let permit = Arc::clone(&page_semaphore);
                        let mut downloader = self.clone();
                        let url = absolute_url.clone();

                        page_tasks.push(tokio::spawn(async move {
                            let _permit = permit.acquire().await;
                            downloader.download_images_inner(&url, max_concurrent_pages).await
                        }));
                    }
                }
            }
        }

        // 等待所有页面任务完成
        for task in page_tasks {
            task.await.map_err(|e| DownloaderError::Other(e.to_string()))??;
        }

        Ok(())
    }

    // 处理页面中的图片
    async fn process_images(&self, document: &Html) -> Result<(), DownloaderError> {
        let img_selector = Selector::parse("img").unwrap();
        let mut image_tasks = vec![];

        for img in document.select(&img_selector) {
            let sources = img.value()
                .attr("src")
                .into_iter()
                .chain(img.value().attr("data-src"))
                .map(String::from)
                .collect::<Vec<_>>();

            for src in sources {
                if let Ok(absolute_url) = self.resolve_url(&src) {
                    if !self.is_already_downloaded(&absolute_url).await && self.is_valid_image_url(&absolute_url) {
                        self.mark_as_downloaded(&absolute_url).await;

                        let permit = Arc::clone(&self.download_semaphore);
                        let client = self.client.clone();
                        let output_dir = self.output_dir.clone();
                        let progress_bar = Arc::clone(&self.progress_bar);

                        image_tasks.push(tokio::spawn(async move {
                            let _permit = permit.acquire().await;
                            match download_image(&client, &absolute_url, &output_dir).await {
                                Ok(_) => progress_bar.inc(1),
                                Err(e) => eprintln!("Failed to download {}: {}", absolute_url, e),
                            }
                        }));
                    }
                }
            }
        }

        // 等待所有图片下载完成
        for task in image_tasks {
            task.await.map_err(|e| DownloaderError::Other(e.to_string()))?;
        }

        Ok(())
    }

    // 检查URL是否已下载
    async fn is_already_downloaded(&self, url: &str) -> bool {
        let downloaded = self.downloaded_images.lock().await;
        downloaded.contains(url)
    }

    // 标记URL为已下载
    async fn mark_as_downloaded(&self, url: &str) {
        let mut downloaded = self.downloaded_images.lock().await;
        downloaded.insert(url.to_string());
    }

    // 解析相对URL为绝对URL
    fn resolve_url(&self, url: &str) -> Result<String, url::ParseError> {
        match Url::parse(url) {
            Ok(absolute) => Ok(absolute.to_string()),
            Err(_) => Ok(self.base_url.join(url)?.to_string()),
        }
    }

    // 检查URL是否属于同一域名
    fn should_visit_url(&self, url: &str) -> bool {
        if let Ok(parsed_url) = Url::parse(url) {
            parsed_url.host_str() == self.base_url.host_str()
        } else {
            false
        }
    }

    // 检查是否为有效的图片URL
    fn is_valid_image_url(&self, url: &str) -> bool {
        let extensions = ["jpg", "jpeg", "png", "gif", "webp"];
        let lower_url = url.to_lowercase();
        extensions.iter().any(|&ext| lower_url.ends_with(ext))
    }
}

// 下载单个图片
async fn download_image(
    client: &Client,
    url: &str,
    output_dir: &str,
) -> Result<PathBuf, DownloaderError> {
    let response = client.get(url).send().await?;

    // 生成唯一文件名
    let file_name = Url::parse(url)?
        .path_segments()
        .and_then(|segments| segments.last())
        .map(|name| name.to_string())
        .unwrap_or_else(|| "image.jpg".to_string());

    let unique_name = format!(
        "{}_{}{}",
        Uuid::new_v4().simple(),
        sanitize_filename::sanitize(&file_name),
        get_extension(&file_name)
    );

    let path = PathBuf::from(output_dir).join(&unique_name);

    let mut file = File::create(&path).await?;
    let bytes = response.bytes().await?;
    file.write_all(&bytes).await?;

    Ok(path)
}

// 获取文件扩展名
fn get_extension(filename: &str) -> String {
    Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!(".{}", ext))
        .unwrap_or_else(|| ".jpg".to_string())
}




#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_download_images() {
        let base_url = "https://tower-survivor.com/v3/index.html";
        let output_dir = "downloaded_images";
        let max_concurrent = 5; // 最大并发下载数
        let max_concurrent_pages = 3;

        let mut downloader = ImageDownloader::new(base_url, output_dir, max_concurrent).unwrap();
        downloader.download_images(base_url, max_concurrent_pages).await.unwrap();

        println!("Download completed!");
    }

}


// 主函数
// #[tokio::main]
// async fn main() -> Result<(), DownloaderError> {
//     let base_url = "https://example.com";
//     let output_dir = "downloaded_images";
//     let max_concurrent_downloads = 5;
//     let max_concurrent_pages = 3;
//
//     let mut downloader = ImageDownloader::new(base_url, output_dir, max_concurrent_downloads)?;
//     downloader.download_images(base_url, max_concurrent_pages).await?;
//
//     println!("Download completed!");
//     Ok(())
// }

