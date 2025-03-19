use futures::future::join_all;
use indicatif::ProgressBar;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::sleep;
use url::Url;

// 错误处理
#[derive(Debug, thiserror::Error)]
pub enum DownloaderError {
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Task join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("Other error: {0}")]
    Other(String),

    #[error("Download error: {0}")]
    Download(String),
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

    fn extract_urls(&self, html: &str) -> (Vec<UrlData>, Vec<String>) {
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

        (image_urls, page_urls)
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
        url: String,
        max_concurrent_pages: usize,
    ) -> Result<(), DownloaderError> {
        fs::create_dir_all(&self.output_dir).await?;
        self.download_images_inner(url, max_concurrent_pages).await
    }

    async fn download_images_inner(
        &self,
        url: String,
        max_concurrent_pages: usize,
    ) -> Result<(), DownloaderError> {
        // 检查是否已访问
        if !self.should_process_url(&url).await {
            return Ok(());
        }

        // 标记为已访问
        {
            let mut visited = self.visited_urls.lock().await;
            visited.insert(url.clone());
        }

        println!("Visiting: {}", url);
        sleep(Duration::from_millis(500)).await;

        // 获取页面内容
        let response = self.client.get(&url).send().await?;
        let html = response.text().await?;

        // 使用 HtmlProcessor 提取 URLs
        let processor = HtmlProcessor::new();
        let (image_urls, page_urls) = processor.extract_urls(&html);

        // 处理图片
        let image_tasks = self.process_images(image_urls).await?;

        // 处理页面链接
        let page_tasks = self.process_pages(page_urls, max_concurrent_pages).await?;

        // 等待所有任务完成
        self.wait_for_tasks(image_tasks, page_tasks).await?;

        Ok(())
    }

    async fn should_process_url(&self, url: &str) -> bool {
        let visited = self.visited_urls.lock().await;
        !visited.contains(url)
    }

    async fn process_images(
        &self,
        image_urls: Vec<UrlData>,
    ) -> Result<Vec<tokio::task::JoinHandle<Result<(), DownloaderError>>>, DownloaderError> {
        let mut tasks = vec![];

        for url_data in image_urls {
            if let Ok(absolute_url) = self.resolve_url(&url_data.url) {
                if !self.is_already_downloaded(&absolute_url).await && self.is_valid_image_url(&absolute_url) {
                    self.mark_as_downloaded(&absolute_url).await;

                    let task = self.spawn_download_task(absolute_url, url_data.alt);
                    tasks.push(task);
                }
            }
        }

        Ok(tasks)
    }

    fn spawn_download_task(
        &self,
        url: String,
        alt: Option<String>,
    ) -> tokio::task::JoinHandle<Result<(), DownloaderError>> {
        let permit = Arc::clone(&self.download_semaphore);
        let client = self.client.clone();
        let output_dir = self.output_dir.clone();
        let progress_bar = Arc::clone(&self.progress_bar);

        tokio::spawn(async move {
            let _permit = permit.acquire().await;
            match download_image(&client, &url, &output_dir).await {
                Ok(_) => {
                    progress_bar.inc(1);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Failed to download {}: {}", url, e);
                    Err(DownloaderError::Download(e.to_string()))
                }
            }
        })
    }

    async fn process_pages(
        &self,
        page_urls: Vec<String>,
        max_concurrent_pages: usize,
    ) -> Result<Vec<tokio::task::JoinHandle<Result<(), DownloaderError>>>, DownloaderError> {
        let mut tasks = vec![];
        let page_semaphore = Arc::new(Semaphore::new(max_concurrent_pages));

        for href in page_urls {
            if let Ok(absolute_url) = self.resolve_url(&href) {
                if self.should_visit_url(&absolute_url) {
                    let permit = Arc::clone(&page_semaphore);
                    let downloader = self.clone();
                    let url = absolute_url.to_string();

                    tasks.push(tokio::spawn(async move {
                        let _permit = permit.acquire().await;
                        downloader.download_images_inner(url, max_concurrent_pages).await
                    }));
                }
            }
        }

        Ok(tasks)
    }

    async fn wait_for_tasks(
        &self,
        image_tasks: Vec<tokio::task::JoinHandle<Result<(), DownloaderError>>>,
        page_tasks: Vec<tokio::task::JoinHandle<Result<(), DownloaderError>>>,
    ) -> Result<(), DownloaderError> {
        // 等待所有图片下载完成
        for task in image_tasks {
            task.await.map_err(|e| DownloaderError::Other(e.to_string()))??;
        }

        // 等待所有页面处理完成
        for task in page_tasks {
            task.await.map_err(|e| DownloaderError::Other(e.to_string()))??;
        }

        Ok(())
    }

    fn resolve_url(&self, url: &str) -> Result<String, url::ParseError> {
        Ok(self.base_url.join(url)?.to_string())
    }

    async fn is_already_downloaded(&self, url: &str) -> bool {
        let downloaded = self.downloaded_images.lock().await;
        downloaded.contains(url)
    }

    async fn mark_as_downloaded(&self, url: &str) {
        let mut downloaded = self.downloaded_images.lock().await;
        downloaded.insert(url.to_string());
    }

    fn is_valid_image_url(&self, url: &str) -> bool {
        let extensions = [".jpg", ".jpeg", ".png", ".gif", ".webp"];
        let url_lower = url.to_lowercase();
        extensions.iter().any(|&ext| url_lower.ends_with(ext))
    }

    fn should_visit_url(&self, url: &str) -> bool {
        url.starts_with(self.base_url.as_str())
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
        DownloaderError::Io(e)
    })?;

    Ok(())
}

// 使用示例
#[cfg(test)]
mod tests {
    use super::*;

    /// 用 #[tokio::test(flavor = "multi_thread")] 注解来运行测试，这样可以更好地暴露并发问题
    #[tokio::test]
    async fn test_download_images() {
        let base_url = "https://tower-survivor.com/v3/index.html";
        let output_dir = "downloaded_images";
        let max_concurrent = 5; // 最大并发下载数
        let max_concurrent_pages = 3;

        let downloader = ImageDownloader::new(base_url, output_dir, max_concurrent).unwrap();
        // downloader.download_images(base_url.to_string(), max_concurrent_pages).await.unwrap();

        println!("Download completed!");


        // 在编译时检查类型
        // assert_is_send::<Arc<Mutex<i32>>>();
        // assert_is_send_sync::<Arc<Mutex<i32>>>();
        //
        // let future = async {
        //     println!("Hello");
        // };
        // assert_future_is_send::<decltype!(future)>();

        assert_send_result(downloader.download_images_inner(base_url.to_string(), max_concurrent));
    }


    // 编译时检查类型是否是 Send
    fn assert_is_send<T: Send>() {}

    // 编译时检查类型是否是 Send + Sync
    fn assert_is_send_sync<T: Send + Sync>() {}

    // 编译时检查 Future 是否是 Send
    fn assert_future_is_send<F: Future + Send>() {}

    // 编译时检查方法是否返回 Send Future
    fn assert_send_result<F>(f: F)
    where
        F: Future<Output = Result<(), DownloaderError>> + Send,
    {
    }

}