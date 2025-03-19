mod ncm_format;
mod ncm_metadata;
mod ncm_music;

use rayon::prelude::*;
use std::fs;
use std::path::Path;

#[cfg(test)]
mod tests {
    use crate::ncm::ncm_music::NeteaseCloudMusic;
    use super::*;

    #[test]
    fn ncm_to_mp3() {
        let dir_path = "/Users/will/Music/网易云音乐";
        let target_path = "/Users/will/Music/mp3";
        let ext_ncm = "ncm";
        let ext_mp3 = "mp3";

        // 获取 NCM 和 MP3 文件列表
        let mut ncm_files = read_dir_files_filter_by_extension(dir_path, ext_ncm);
        let mp3_files: Vec<String> = read_dir_files_filter_by_extension(target_path, ext_mp3)
            .into_iter()
            .map(|f| f.replace(ext_mp3, ext_ncm))
            .collect();

        println!("MP3 files (renamed): {:?}", mp3_files);

        // 排除同名文件
        ncm_files.retain(|ncm| !mp3_files.contains(ncm));

        if ncm_files.is_empty() {
            println!("No NCM files found");
            return;
        }

        println!("NCM files to process: {:?}", ncm_files);

        // 并行处理文件
        let processed_count = ncm_files
            .par_iter() // 使用 rayon 的并行迭代器
            .map(|file_name| {
                let file_path = Path::new(dir_path).join(file_name);
                println!("Processing file: {:?}", file_path);

                match NeteaseCloudMusic::new(file_path.to_str().unwrap()) {
                    Ok(mut ncm) => {
                        if let Err(e) = ncm.dump(Some(target_path)) {
                            eprintln!("Failed to dump {}: {}", file_name, e);
                            return Err(());
                        }
                        if let Err(e) = ncm.fix_metadata(true) {
                            eprintln!("Failed to fix metadata for {}: {}", file_name, e);
                            return Err(());
                        }
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("Failed to open {}: {}", file_name, e);
                        Err(())
                    }
                }
            })
            .filter(|result| result.is_ok()) // 统计成功处理的文件
            .count();

        println!("DONE! Successfully processed {} NCM files", processed_count);

    }
}


fn read_dir_files(dir_path: &str) -> Vec<String> {
    fs::read_dir(dir_path)
        .unwrap()
        .filter_map(|x| {
            x.ok()
                .and_then(|x| Some(x.file_name().to_string_lossy().to_string()))
        })
        .collect()
}

fn read_dir_files_filter_by_extension(path: &str, ext: &str) -> Vec<String> {
    fs::read_dir(path)
        .unwrap()
        .filter_map(|x| {
            x.ok().and_then(|e| {
                let path = e.path();
                if (path.is_file() && path.extension().and_then(|s| s.to_str()) == Some(ext)) {
                    path.file_name()
                        .and_then(|s| Some(s.to_string_lossy().to_string()))
                } else {
                    None
                }
            })
        })
        .collect()
}
