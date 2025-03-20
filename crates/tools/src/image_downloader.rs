use futures::stream::{self, StreamExt};
use indicatif::ProgressBar;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use std::time::Duration;
use url::Url;

// 错误类型
#[derive(Debug, thiserror::Error)]
pub enum DownloaderError {
    // #[error("Request error: {0}")]
    // Request(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Task join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("Other error: {0}")]
    Other(String),
    #[error("Download error: {0}")]
    Download(#[from] reqwest::Error),
}

// URL 数据结构
#[derive(Debug, Clone)]
struct UrlData {
    url: String,
    alt: Option<String>,
}

impl UrlData {
    fn new(url: String, alt: Option<String>) -> Self {
        Self { url, alt }
    }
}

// 提取的 URLs 结构
#[derive(Debug)]
struct ExtractedUrls {
    image_urls: Vec<UrlData>,
    page_urls: Vec<String>,
}

// HTML 处理器
struct HtmlProcessor {
    img_selector: Selector,
    link_selector: Selector,
}

impl HtmlProcessor {
    fn new() -> Self {
        Self {
            img_selector: Selector::parse("img").unwrap(),
            link_selector: Selector::parse("a").unwrap(),
        }
    }

    fn process_html(&self, html: &str) -> ExtractedUrls {
        let document = Html::parse_document(html);

        // 提取图片 URL
        let image_urls = document
            .select(&self.img_selector)
            .flat_map(|img| {
                let mut urls = Vec::new();
                let alt = img.value().attr("alt").map(String::from);

                if let Some(src) = img.value().attr("src") {
                    urls.push(UrlData::new(src.to_string(), alt.clone()));
                }
                if let Some(data_src) = img.value().attr("data-src") {
                    urls.push(UrlData::new(data_src.to_string(), alt.clone()));
                }
                urls
            })
            .collect();

        // 提取页面链接
        let page_urls = document
            .select(&self.link_selector)
            .filter_map(|link| link.value().attr("href"))
            .map(String::from)
            .collect();

        ExtractedUrls {
            image_urls,
            page_urls,
        }
    }
}

// 下载器
#[derive(Clone)]
pub struct ImageDownloader {
    client: Client,
    base_url: Url,
    visited_urls: Arc<Mutex<HashSet<String>>>,
    downloaded_images: Arc<Mutex<HashSet<String>>>,
    output_dir: String,
    download_semaphore: Arc<Semaphore>,
    progress_bar: Arc<ProgressBar>,
}

impl ImageDownloader {
    pub fn new(
        base_url: &str,
        output_dir: &str,
        max_concurrent_downloads: usize,
    ) -> Result<Self, DownloaderError> {
        Ok(Self {
            client: Client::new(),
            base_url: Url::parse(base_url)?,
            visited_urls: Arc::new(Mutex::new(HashSet::new())),
            downloaded_images: Arc::new(Mutex::new(HashSet::new())),
            output_dir: output_dir.to_string(),
            download_semaphore: Arc::new(Semaphore::new(max_concurrent_downloads)),
            progress_bar: Arc::new(ProgressBar::new(0)),
        })
    }

    pub async fn download_images(
        &self,
        start_url: String,
        max_concurrent_pages: usize,
    ) -> Result<(), DownloaderError> {
        fs::create_dir_all(&self.output_dir).await?;

        let mut page_queue = VecDeque::new();
        page_queue.push_back(start_url);

        while let Some(url) = page_queue.pop_front() {
            if !self.should_process_url(&url).await {
                continue;
            }

            // 标记为已访问
            self.mark_visited(&url).await;

            // 获取并解析页面
            let (image_urls, page_urls) = self.fetch_and_parse_page(&url).await?;

            // 处理发现的新页面
            for url in page_urls {
                if let Ok(absolute_url) = self.resolve_url(&url) {
                    if self.should_visit_url(&absolute_url) {
                        page_queue.push_back(absolute_url);
                    }
                }
            }

            // 下载图片
            let download_tasks = self.spawn_download_tasks(image_urls);

            // 等待当前页面的图片下载完成
            for task in download_tasks {
                task.await.map_err(|e| DownloaderError::Other(e.to_string()))??;
            }
        }

        Ok(())
    }

    async fn fetch_and_parse_page(&self, url: &str) -> Result<(Vec<UrlData>, Vec<String>), DownloaderError> {
        println!("Visiting: {}", url);
        sleep(Duration::from_millis(500)).await;

        let response = self.client.get(url).send().await?;
        let html = response.text().await?;

        // 同步处理 HTML
        let processor = HtmlProcessor::new();
        let extracted = processor.process_html(&html);

        Ok((extracted.image_urls, extracted.page_urls))
    }

    fn spawn_download_tasks(&self, image_urls: Vec<UrlData>) -> Vec<JoinHandle<Result<(), DownloaderError>>> {
        image_urls
            .into_iter()
            .filter_map(|url_data| {
                self.resolve_url(&url_data.url)
                    .ok()
                    .filter(|url| self.is_valid_image_url(url))
                    .map(|url| {
                        let client = self.client.clone();
                        let output_dir = self.output_dir.clone();
                        let semaphore = Arc::clone(&self.download_semaphore);
                        let progress_bar = Arc::clone(&self.progress_bar);

                        tokio::spawn(async move {
                            let _permit = semaphore.acquire().await;
                            match download_image(&client, &url, &output_dir).await {
                                Ok(_) => {
                                    progress_bar.inc(1);
                                    Ok(())
                                }
                                Err(e) => {
                                    eprintln!("Failed to download {}: {}", url, e);
                                    Err(e)
                                }
                            }
                        })
                    })
            })
            .collect()
    }

    async fn should_process_url(&self, url: &str) -> bool {
        let visited = self.visited_urls.lock().await;
        !visited.contains(url)
    }

    async fn mark_visited(&self, url: &str) {
        let mut visited = self.visited_urls.lock().await;
        visited.insert(url.to_string());
    }

    async fn is_already_downloaded(&self, url: &str) -> bool {
        let downloaded = self.downloaded_images.lock().await;
        downloaded.contains(url)
    }

    async fn mark_as_downloaded(&self, url: &str) {
        let mut downloaded = self.downloaded_images.lock().await;
        downloaded.insert(url.to_string());
    }

    fn resolve_url(&self, url: &str) -> Result<String, url::ParseError> {
        Ok(self.base_url.join(url)?.to_string())
    }

    fn should_visit_url(&self, url: &str) -> bool {
        url.starts_with(self.base_url.as_str())
    }

    fn is_valid_image_url(&self, url: &str) -> bool {
        let extensions = [".jpg", ".jpeg", ".png", ".gif", ".webp"];
        let url_lower = url.to_lowercase();
        extensions.iter().any(|&ext| url_lower.ends_with(ext))
    }
}

async fn download_image(
    client: &Client,
    url: &str,
    output_dir: &str,
) -> Result<(), DownloaderError> {
    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;

    let file_name = url
        .split('/')
        .last()
        .unwrap_or("image.jpg")
        .split('?')
        .next()
        .unwrap_or("image.jpg");

    let path = Path::new(output_dir).join(file_name);
    fs::write(&path, &bytes).await.map_err(|e| {
        eprintln!("Failed to write file: {}", e);
        DownloaderError::Other(e.to_string())
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_download_images() -> Result<(), DownloaderError> {
        let base_url = "https://tower-survivor.com/v3/index.html";
        let base_url = "https://igoutu.cn/icons/new";
        let base_url = "https://icons8.com/icons/set/avatar";
        let output_dir = "downloaded_images/icons";
        let max_concurrent_downloads = 5;
        let max_concurrent_pages = 3;

        let downloader = ImageDownloader::new(base_url, output_dir, max_concurrent_downloads)?;
        downloader.download_images(base_url.to_string(), max_concurrent_pages).await?;

        println!("Download completed!");
        Ok(())
    }

}