use super::{ncm_format::NcmFormat, ncm_metadata::NcmMetadata};
use id3::{Frame, Tag, TagLike};
use reqwest;

use id3::frame::{Picture, PictureType};
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use common::aes_des_utils;

pub struct NeteaseCloudMusic {
    s_core_key: [u8; 17],
    s_modify_key: [u8; 17],
    m_png: [u8; 8],

    m_file_path: String,

    m_dump_file_path: String,
    m_format: NcmFormat,
    m_image_data: Vec<u8>,
    m_file_stream: Option<File>,
    m_key_box: [u8; 256],
    m_metadata: Option<NcmMetadata>,
}

impl NeteaseCloudMusic {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let mut ncm = NeteaseCloudMusic {
            s_core_key: [
                0x68, 0x7A, 0x48, 0x52, 0x41, 0x6D, 0x73, 0x6F, 0x35, 0x6B, 0x49, 0x6E, 0x62, 0x61,
                0x78, 0x57, 0,
            ],
            s_modify_key: [
                0x23, 0x31, 0x34, 0x6C, 0x6A, 0x6B, 0x5F, 0x21, 0x5C, 0x5D, 0x26, 0x30, 0x55, 0x3C,
                0x27, 0x28, 0,
            ],
            m_png: [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
            m_file_path: file_path.to_string(),
            m_dump_file_path: "".to_string(),
            m_format: NcmFormat::MP3,
            m_image_data: vec![],
            m_file_stream: None,
            m_key_box: [0u8; 256],
            m_metadata: None,
            // m_album_pic_url: "".to_string(),
        };

        // 打开文件
        if !ncm.open_file()? {
            return Err("Failed to open file.".into());
        }

        // 检查文件是否为 NCM 文件
        if !ncm.check_ncm_file()? {
            return Err("Not an NCM file.".into());
        }

        // 跳过版本信息（2字节）
        ncm.m_file_stream
            .as_mut()
            .unwrap()
            .seek(SeekFrom::Current(2))?;

        // 读取 RC4 密钥长度
        let mut key_len_buf = [0u8; 4];
        ncm.read(&mut key_len_buf)?;

        let key_len = u32::from_le_bytes(key_len_buf) as usize;

        // 读取 RC4 密钥
        let mut key_data = vec![0u8; key_len];
        ncm.read(&mut key_data)?;

        // 对密钥数据进行解密
        for byte in key_data.iter_mut() {
            *byte ^= 0x64;
        }
        // let key_data_str = aes_des_utils::encode_base64(key_data)?;
        let decrypted_key = aes_des_utils::aes_decrypt_bytes(&ncm.s_core_key[..16], &key_data);
        let decrypted_key_bytes = decrypted_key.as_bytes();
        // let decrypted_key = utils::aes_ecb_decrypt_new(&ncm.s_core_key[..16], &key_data);
        // 构建 RC4 密钥盒
        ncm.build_key_box(&decrypted_key_bytes[17..]);

        // 读取元数据长度
        let mut metadata_len_buf = [0u8; 4];
        ncm.read(&mut metadata_len_buf)?;

        let metadata_len = u32::from_le_bytes(metadata_len_buf) as usize;

        if metadata_len > 0 {
            // 读取并解密元数据
            let mut metadata_data = vec![0u8; metadata_len];
            ncm.read(&mut metadata_data)?;

            for byte in metadata_data.iter_mut() {
                *byte ^= 0x63;
            }

            // 解密元数据
            let swap_data = String::from_utf8_lossy(&metadata_data[22..]);

            let decrypted_metadata =
                aes_des_utils::aes_decrypt(&ncm.s_modify_key[..16], &swap_data);

            println!("Decrypted metadata: {:?}", &decrypted_metadata);

            let decrypted_metadata_bytes = decrypted_metadata.as_bytes();

            println!(
                "Decrypted metadata: {:?}",
                &String::from_utf8_lossy(&decrypted_metadata_bytes[6..])
            );

            // 提取专辑封面 URL
            // ncm.m_album_pic_url =
            //     ncm.get_album_pic_url(&String::from_utf8_lossy(&decrypted_metadata[6..]));
            ncm.m_metadata = Some(NcmMetadata::new(&String::from_utf8_lossy(
                &decrypted_metadata_bytes[6..],
            )));
            // if let Some(metadata) = ncm.m_metadata {
            //     ncm.m_album_pic_url = metadata.m_album_pic_url.clone()
            // }
        } else {
            // 如果没有元数据，设为 None
            ncm.m_metadata = None;
        }

        // 跳过 5 字节间隙
        ncm.m_file_stream
            .as_mut()
            .unwrap()
            .seek(SeekFrom::Current(5))?;

        // 读取封面帧的长度
        let mut cover_frame_len_buf = [0u8; 4];
        ncm.read(&mut cover_frame_len_buf)?;

        // 读取封面数据长度
        let mut cover_data_len_buf = [0u8; 4];
        ncm.read(&mut cover_data_len_buf)?;

        let cover_frame_len = u32::from_le_bytes(cover_frame_len_buf) as usize;
        let cover_data_len = u32::from_le_bytes(cover_data_len_buf) as usize;

        // 读取封面图像数据
        if cover_data_len > 0 {
            let mut cover_data = vec![0u8; cover_data_len];
            ncm.read(&mut cover_data)?;
            ncm.m_image_data = cover_data;
        }

        // 跳过封面帧的剩余数据
        ncm.m_file_stream.as_mut().unwrap().seek(SeekFrom::Current(
            cover_frame_len as i64 - cover_data_len as i64,
        ))?;

        Ok(ncm)
    }

    fn open_file(&mut self) -> Result<bool, Box<dyn Error>> {
        self.m_file_stream = Some(File::open(&self.m_file_path)?);
        Ok(true)
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.m_file_stream.as_mut().unwrap().read(buffer)
    }

    pub fn check_ncm_file(&mut self) -> Result<bool, Box<dyn Error>> {
        let mut header = [0u8; 8];

        // if let Some(stream) = &mut self.m_file_stream {
        //     stream.read_exact(&mut header)?;
        //     return Ok(&header[0..4] == b"CTEN" && &header[4..8] == b"FDAM");
        // }
        // Ok(false)

        self.read(&mut header)?;
        Ok(&header[0..4] == b"CTEN" && &header[4..8] == b"FDAM")
    }

    /// Build the RC4 key box
    fn build_key_box(&mut self, key: &[u8]) {
        for i in 0..256 {
            self.m_key_box[i] = i as u8;
        }

        let mut j = 0;
        for i in 0..256 {
            j = (j + self.m_key_box[i] as usize + key[i % key.len()] as usize) % 256;
            self.m_key_box.swap(i, j);
        }
    }

    pub fn dump(&mut self, target_dir: Option<&str>) -> Result<(), Box<dyn Error>> {
        // 保留原文件路径作为转换目标
        self.m_dump_file_path = self.m_file_path.clone();
        // 准备缓冲区
        let mut buffer = vec![0; 0x8000];
        let mut find_format_flag = false;
        let mut output_stream = None;

        loop {
            // 读取数据
            let n = self.read(&mut buffer)?;
            if n == 0 {
                break;
            }

            // 解密缓冲区
            for i in 0..n {
                let j = (i + 1) & 0xff;
                // buffer[i] ^= self.m_key_box[(self.m_key_box[j as usize] + self.m_key_box[(self.m_key_box[j as usize] + j) & 0xff]) as usize & 0xff];
                buffer[i] ^= self.m_key_box[(self.m_key_box[j] as usize
                    + self.m_key_box[(self.m_key_box[j] as usize + j) & 0xFF] as usize)
                    & 0xFF];
            }

            // 首次读取时确定文件格式
            if !find_format_flag {
                self.m_format = if buffer[0] == 0x49 && buffer[1] == 0x44 && buffer[2] == 0x33 {
                    NcmFormat::MP3
                } else {
                    NcmFormat::FLAC
                };

                // 更改文件扩展名
                let new_ext = match self.m_format {
                    NcmFormat::MP3 => ".mp3",
                    NcmFormat::FLAC => ".flac",
                };

                self.m_dump_file_path = self.m_dump_file_path.replace(".ncm", new_ext);

                if let Some(target) = target_dir {
                    self.m_dump_file_path = format!(
                        "{}/{}",
                        target,
                        Path::new(&self.m_dump_file_path)
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                    );
                }
                // 创建输出文件
                output_stream = Some(BufWriter::new(File::create(&self.m_dump_file_path)?));
                find_format_flag = true;
            }
            // 写入解密后的数据
            if let Some(output) = output_stream.as_mut() {
                output.write_all(&buffer[..n])?;
            }
        }
        Ok(())
    }

    // 修复元数据
    pub fn fix_metadata(
        &mut self,
        fetch_album_image_from_remote: bool,
    ) -> Result<(), Box<dyn Error>> {
        if self.m_image_data.is_empty() && fetch_album_image_from_remote {
            // 从远程获取专辑封面
            let response =
                reqwest::blocking::get(&self.m_metadata.as_ref().unwrap().m_album_pic_url)?;
            if response.status().is_success() {
                self.m_image_data = response.bytes()?.to_vec();
            }
        }

        match self.m_format {
            NcmFormat::MP3 => {
                let mut tag = Tag::read_from_path(&self.m_dump_file_path)?;
                tag.set_title(&self.m_metadata.as_ref().unwrap().m_name);
                tag.set_artist(&self.m_metadata.as_ref().unwrap().m_artist);
                tag.set_album(&self.m_metadata.as_ref().unwrap().m_album);

                if !self.m_image_data.is_empty() {
                    let frame = Frame::from(Picture {
                        mime_type: "".to_string(),
                        description: "Front Cover".to_string(),
                        data: self.m_image_data.clone(),
                        picture_type: PictureType::CoverFront,
                    });
                    tag.add_frame(frame);
                }
                tag.write_to_path(&self.m_dump_file_path, id3::Version::Id3v24)?;
            }
            // Some(NcmFormat::FLAC) => {
            //     let mut flac = FlacFile::open(&self.dump_file_path)?;
            //     if !self.image_data.is_empty() {
            //         let pic = picture::Picture::from_data(self.image_data.clone())?;
            //         flac.add_picture(pic);
            //     }
            //
            //     let mut metadata = HashMap::new();
            //     metadata.insert("TITLE", self.metadata.as_ref().unwrap().name.clone());
            //     metadata.insert("ARTIST", self.metadata.as_ref().unwrap().artist.clone());
            //     metadata.insert("ALBUM", self.metadata.as_ref().unwrap().album.clone());
            //
            //     flac.set_metadata(metadata)?;
            //     flac.save()?;
            // }
            _ => {}
        }
        Ok(())
    }

    // 获取解密后的文件路径
    pub fn get_dump_file_path(&self) -> &str {
        &self.m_dump_file_path
    }

    pub fn build_key_box_v1(&mut self, key: &[u8]) {
        let key_len = key.len();
        if key_len == 0 {
            panic!("Key length must be greater than zero");
        }

        // Initialize m_key_box with values from 0 to 255
        for i in 0..256 {
            self.m_key_box[i] = i as u8;
        }

        let mut swap: u8;
        let mut last_byte: u8 = 0;
        let mut key_offset: usize = 0;

        for i in 0..256 {
            swap = self.m_key_box[i];
            let c = (swap as u16 + last_byte as u16 + key[key_offset] as u16) & 0xff;
            let c = c as usize;

            key_offset += 1;
            if key_offset >= key_len {
                key_offset = 0;
            }

            // Swap values in m_key_box
            self.m_key_box[i] = self.m_key_box[c];
            self.m_key_box[c] = swap;
            last_byte = c as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_key_box() {
        let file_path = "../assets/周传雄 - 青花.ncm";
        let mut ncm = NeteaseCloudMusic::new(file_path).unwrap();
        let key = b"examplekey";
        // Print the initialized m_key_box for verification
        ncm.build_key_box(key);
        println!("origin:{:?}", ncm.m_key_box);

        ncm.build_key_box_v1(key);
        println!("v1: {:?}", ncm.m_key_box);

        assert_eq!(ncm.build_key_box(key), ncm.build_key_box_v1(key));
    }

    #[test]
    fn test_le() {
        let hex_value: u32 = 0x4E455443;

        println!("{}", hex_value);
        println!("{:?}", b"CTEN");
        // 按小端字节序拆解为 [0x4E, 0x45, 0x54, 0x43]
        let bytes = hex_value.to_le_bytes();
        println!(
            "按小端字节序拆解为：{:?}, {:X?}  转为字符串：{:?}",
            bytes,
            bytes,
            String::from_utf8_lossy(&bytes).into_owned()
        );
        // 按大端字节序拆解为 [0x4E, 0x45, 0x54, 0x43]
        let bytes = hex_value.to_be_bytes();
        println!(
            "按大端字节序拆解为：{:?}, {:X?}  转为字符串：{:?}",
            bytes,
            bytes,
            String::from_utf8_lossy(&bytes).into_owned()
        );
    }
}
