use wasm_bindgen::prelude::*;
use image::{ImageBuffer, Rgba, ColorType, ImageEncoder};
use base64::{Engine as _, engine::general_purpose};
use std::io::Cursor;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct MediaConverter;

#[wasm_bindgen]
impl MediaConverter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> MediaConverter {
        MediaConverter
    }

    /// 将文件字节数据转换为PNG图像数据URL
    #[wasm_bindgen]
    pub fn file_bytes_to_image_data_url(file_bytes: &[u8], format: &str) -> String {
        // 计算图像尺寸
        let data_len = file_bytes.len();
        
        // 添加元数据：前4个字节存储原始文件大小，接下来4个字节存储格式信息长度，再接下来是格式信息
        let metadata_size = 4;
        let format_bytes = format.as_bytes();
        let format_len = format_bytes.len();
        let format_header_size = 4 + format_len; // 4字节存储格式长度 + 格式字符串
        let total_data_len = data_len + metadata_size + format_header_size;
        
        // 每个像素可以存储4字节（RGBA）
        let pixels_needed = (total_data_len + 3) / 4;
        let sqrt_pixels = (pixels_needed as f64).sqrt().ceil() as u32;
        
        // 创建正方形图像
        let width = sqrt_pixels;
        let height = sqrt_pixels;
        
        // 创建图像缓冲区
        let mut img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);
        
        // 将元数据写入图像
        let mut byte_index = 0;
        
        // 写入文件大小（4字节）
        let data_size_bytes = (data_len as u32).to_le_bytes();
        for i in 0..4 {
            if byte_index < width * height * 4 {
                let pixel_index = byte_index / 4;
                let channel_index = byte_index % 4;
                let x = (pixel_index as u32) % width;
                let y = (pixel_index as u32) / width;
                let mut pixel = img_buffer.get_pixel(x, y).clone();
                pixel[channel_index.try_into().unwrap()] = data_size_bytes[i];
                img_buffer.put_pixel(x, y, pixel);
                byte_index += 1;
            }
        }
        
        // 写入格式信息长度（4字节）
        let format_len_bytes = (format_len as u32).to_le_bytes();
        for i in 0..4 {
            if byte_index < width * height * 4 {
                let pixel_index = byte_index / 4;
                let channel_index = byte_index % 4;
                let x = (pixel_index as u32) % width;
                let y = (pixel_index as u32) / width;
                let mut pixel = img_buffer.get_pixel(x, y).clone();
                pixel[channel_index.try_into().unwrap()] = format_len_bytes[i];
                img_buffer.put_pixel(x, y, pixel);
                byte_index += 1;
            }
        }
        
        // 写入格式信息
        for &byte in format_bytes {
            if byte_index < width * height * 4 {
                let pixel_index = byte_index / 4;
                let channel_index = byte_index % 4;
                let x = (pixel_index as u32) % width;
                let y = (pixel_index as u32) / width;
                let mut pixel = img_buffer.get_pixel(x, y).clone();
                pixel[channel_index.try_into().unwrap()] = byte;
                img_buffer.put_pixel(x, y, pixel);
                byte_index += 1;
            }
        }
        
        // 写入文件数据
        for &byte in file_bytes {
            if byte_index < width * height * 4 {
                let pixel_index = byte_index / 4;
                let channel_index = byte_index % 4;
                let x = (pixel_index as u32) % width;
                let y = (pixel_index as u32) / width;
                let mut pixel = img_buffer.get_pixel(x, y).clone();
                pixel[channel_index.try_into().unwrap()] = byte;
                img_buffer.put_pixel(x, y, pixel);
                byte_index += 1;
            }
        }
        
        // 用0填充剩余空间
        while byte_index < width * height * 4 {
            let pixel_index = byte_index / 4;
            let channel_index = byte_index % 4;
            let x = (pixel_index as u32) % width;
            let y = (pixel_index as u32) / width;
            let mut pixel = img_buffer.get_pixel(x, y).clone();
            pixel[channel_index.try_into().unwrap()] = 0;
            img_buffer.put_pixel(x, y, pixel);
            byte_index += 1;
        }
        
        // 将图像编码为指定格式
        let mut image_data: Vec<u8> = Vec::new();
        match format {
            "png" => {
                let encoder = image::codecs::png::PngEncoder::new(&mut image_data);
                encoder.write_image(&img_buffer, width, height, ColorType::Rgba8)
                    .expect("Failed to encode PNG image");
            },
            "bmp" => {
                let mut encoder = image::codecs::bmp::BmpEncoder::new(&mut image_data);
                encoder.encode(&img_buffer, width, height, ColorType::Rgba8)
                    .expect("Failed to encode BMP image");
            },
            "jpeg" => {
                let mut encoder = image::codecs::jpeg::JpegEncoder::new(&mut image_data);
                encoder.encode(&img_buffer, width, height, ColorType::Rgba8)
                    .expect("Failed to encode JPEG image");
            },
            _ => {
                // 默认使用PNG
                let encoder = image::codecs::png::PngEncoder::new(&mut image_data);
                encoder.write_image(&img_buffer, width, height, ColorType::Rgba8)
                    .expect("Failed to encode PNG image");
            }
        }
        
        // 将图像数据转换为base64数据URL
        let base64_data = general_purpose::STANDARD.encode(&image_data);
        format!("data:image/{};base64,{}", format, base64_data)
    }
    
    /// 从图像数据URL提取文件字节数据
    #[wasm_bindgen]
    pub fn image_data_url_to_file_bytes(data_url: &str) -> Vec<u8> {
        // 移除数据URL前缀
        let parts: Vec<&str> = data_url.split(',').collect();
        if parts.len() < 2 {
            eprintln!("Invalid data URL format");
            return Vec::new();
        }
        let base64_data = parts[1];
        
        // 解码base64数据
        let image_data = match general_purpose::STANDARD.decode(base64_data) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to decode base64 data: {}", e);
                return Vec::new();
            }
        };
        
        // 解码图像
        let img = match image::load_from_memory(&image_data) {
            Ok(img) => img,
            Err(e) => {
                eprintln!("Failed to load image from data: {}", e);
                return Vec::new();
            }
        };
        
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();
        
        // 检查图像是否足够大以包含元数据
        if width * height * 4 < 8 {
            eprintln!("Image too small to contain metadata");
            return Vec::new();
        }
        
        // 从像素中读取元数据
        let mut byte_index = 0;
        
        // 读取文件大小（4字节）
        let mut size_bytes = [0u8; 4];
        for i in 0..4 {
            let pixel_index = byte_index / 4;
            let channel_index = byte_index % 4;
            let x = (pixel_index as u32) % width;
            let y = (pixel_index as u32) / width;
            size_bytes[i] = rgba_img.get_pixel(x, y)[channel_index];
            byte_index += 1;
        }
        let file_size = u32::from_le_bytes(size_bytes) as usize;
        
        // 验证文件大小是否合理
        if file_size == 0 || file_size > 100 * 1024 * 1024 { // 限制为100MB
            eprintln!("Invalid file size: {}", file_size);
            return Vec::new();
        }
        
        // 读取格式信息长度（4字节）
        let mut format_len_bytes = [0u8; 4];
        for i in 0..4 {
            let pixel_index = byte_index / 4;
            let channel_index = byte_index % 4;
            let x = (pixel_index as u32) % width;
            let y = (pixel_index as u32) / width;
            format_len_bytes[i] = rgba_img.get_pixel(x, y)[channel_index];
            byte_index += 1;
        }
        let format_len = u32::from_le_bytes(format_len_bytes) as usize;
        
        // 验证格式长度是否合理
        if format_len > 100 {
            eprintln!("Invalid format length: {}", format_len);
            return Vec::new();
        }
        
        // 跳过格式信息
        byte_index += format_len;
        
        // 检查是否有足够的数据
        if byte_index + file_size > (width * height * 4) as usize {
            eprintln!("Image does not contain enough data for file reconstruction");
            return Vec::new();
        }
        
        // 从图像中提取文件数据
        let mut file_bytes: Vec<u8> = Vec::with_capacity(file_size);
        
        for _ in 0..file_size {
            if byte_index < (width * height * 4) as usize {
                let pixel_index = byte_index / 4;
                let channel_index = byte_index % 4;
                let x = (pixel_index as u32) % width;
                let y = (pixel_index as u32) / width;
                file_bytes.push(rgba_img.get_pixel(x, y)[channel_index]);
                byte_index += 1;
            }
        }
        
        file_bytes
    }
    
    /// 将文件字节数据转换为WAV音频数据URL
    #[wasm_bindgen]
    pub fn file_bytes_to_audio_data_url(file_bytes: &[u8]) -> String {
        // 音频配置参数
        const METADATA_SIZE: usize = 4; // 4字节用于存储文件大小
        const SAMPLE_RATE: u32 = 44100;
        const CHANNELS: u16 = 1;
        const BITS_PER_SAMPLE: u16 = 16;
        
        // 获取文件大小并准备元数据
        let data_len = file_bytes.len();
        let data_size_bytes = (data_len as u32).to_le_bytes();
        
        // 创建WAV样本数据（16位）
        let mut samples: Vec<i16> = Vec::with_capacity(data_len + METADATA_SIZE);
        
        // 写入元数据（文件大小）
        for &byte in &data_size_bytes {
            samples.push(byte as i16);
        }
        
        // 写入文件数据
        for &byte in file_bytes {
            samples.push(byte as i16);
        }
        
        // 创建WAV规格
        let spec = hound::WavSpec {
            channels: CHANNELS,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: BITS_PER_SAMPLE,
            sample_format: hound::SampleFormat::Int,
        };
        
        // 创建WAV数据
        let mut wav_data = Vec::new();
        {
            let cursor = Cursor::new(&mut wav_data);
            let mut writer = hound::WavWriter::new(cursor, spec).expect("Failed to create WAV writer");
            for sample in samples {
                writer.write_sample(sample).expect("Failed to write sample");
            }
            writer.finalize().expect("Failed to finalize WAV");
        }
        
        // 将WAV数据转换为base64数据URL
        let base64_data = general_purpose::STANDARD.encode(&wav_data);
        format!("data:audio/wav;base64,{}", base64_data)
    }
    
    /// 从音频数据URL提取文件字节数据
    #[wasm_bindgen]
    pub fn audio_data_url_to_file_bytes(data_url: &str) -> Vec<u8> {
        // 移除数据URL前缀
        let parts: Vec<&str> = data_url.split(',').collect();
        if parts.len() < 2 {
            eprintln!("Invalid data URL format");
            return Vec::new();
        }
        let base64_data = parts[1];
        
        // 解码base64数据
        let wav_data = match general_purpose::STANDARD.decode(base64_data) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to decode base64 data: {}", e);
                return Vec::new();
            }
        };
        
        // 解码WAV音频
        let cursor = Cursor::new(wav_data);
        let mut reader = match hound::WavReader::new(cursor) {
            Ok(reader) => reader,
            Err(e) => {
                eprintln!("Failed to load audio from WAV data: {}", e);
                return Vec::new();
            }
        };
        
        // 读取所有样本
        let samples: Vec<i16> = reader.samples()
            .map(|s| s.unwrap_or(0))
            .collect();
        
        if samples.len() < 4 {
            eprintln!("Audio data too short to contain metadata");
            return Vec::new();
        }
        
        // 从样本中读取元数据（原始文件大小）
        let file_size = u32::from_le_bytes([
            samples[0] as u8, 
            samples[1] as u8, 
            samples[2] as u8, 
            samples[3] as u8
        ]) as usize;
        
        // 验证文件大小是否合理
        if file_size == 0 || file_size > 100 * 1024 * 1024 { // 限制为100MB
            eprintln!("Invalid file size: {}", file_size);
            return Vec::new();
        }
        
        if file_size == 0 || samples.len() < 4 + file_size {
            eprintln!("Invalid file size or incomplete audio data");
            return Vec::new();
        }
        
        // 从样本中提取文件数据
        let mut file_bytes: Vec<u8> = Vec::with_capacity(file_size);
        
        // 从第五个样本开始读取文件数据
        for i in 4..4 + file_size {
            if i < samples.len() {
                file_bytes.push(samples[i] as u8);
            } else {
                eprintln!("Incomplete data in audio file");
                return Vec::new();
            }
        }
        
        file_bytes
    }
}