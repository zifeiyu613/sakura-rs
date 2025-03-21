use crate::downloader::any_downloader::ImageDownloader;
use crate::downloader::error::DownloaderError;

mod ncm;
mod downloader;





/// ```
/// cargo run -- image_urls.txt downloaded_images 10
/// ```
#[tokio::main]
async fn main() -> Result<(), DownloaderError> {
    let args: Vec<String> = std::env::args().collect();

    download(args).await?;

    Ok(())
}


async fn download(args: Vec<String>) -> Result<(), DownloaderError> {

    if args.len() < 2 {
        eprintln!("Usage: {} <url_file> [output_dir] [concurrent_downloads]", args[0]);
        eprintln!("  url_file: Path to text file with image URLs");
        eprintln!("  output_dir: Directory to save images (default: downloaded_images)");
        eprintln!("  concurrent_downloads: Max concurrent downloads (default: 5)");
        return Ok(());
    }

    let file_path = &args[1];
    let output_dir = args.get(2).map_or("./downloaded_images", |s| s);
    let max_concurrent = args.get(3)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(5);

    println!("Starting download from file: {}", file_path);
    println!("Output directory: {}", output_dir);
    println!("Max concurrent downloads: {}", max_concurrent);

    let downloader = ImageDownloader::new(output_dir, max_concurrent);
    downloader.download_from_file(file_path).await?;

    Ok(())
}
