use base64::Engine;
use regex::Regex;
use std::fs;

// 从Markdown文本中提取图片数据
fn extract_image_from_markdown(text: &str) -> Option<(String, Vec<u8>)> {
    // 匹配 ![image](data:image/png;base64,xxxxx) 格式
    let re = Regex::new(r"!\[.*?\]\(data:(image/[a-zA-Z]+);base64,([A-Za-z0-9+/=]+)\)").ok()?;

    if let Some(captures) = re.captures(text) {
        let mime_type = captures.get(1)?.as_str().to_string();
        let base64_data = captures.get(2)?.as_str();

        // 解码base64数据
        if let Ok(image_bytes) = base64::engine::general_purpose::STANDARD.decode(base64_data) {
            return Some((mime_type, image_bytes));
        }
    }

    None
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 读取之前保存的响应文件
    let response_text = fs::read_to_string("dog_generation_response.txt")?;

    println!("正在从响应文件中提取图片...");

    if let Some((mime_type, image_bytes)) = extract_image_from_markdown(&response_text) {
        let image_size = image_bytes.len(); // 先获取大小
        let file_extension = if mime_type.contains("png") {
            "png"
        } else if mime_type.contains("jpeg") || mime_type.contains("jpg") {
            "jpg"
        } else if mime_type.contains("gif") {
            "gif"
        } else if mime_type.contains("webp") {
            "webp"
        } else {
            "png"
        };

        let output_filename = format!("generated_dog_from_markdown.{}", file_extension);
        fs::write(&output_filename, &image_bytes)?;
        println!("成功！从Markdown中提取的狗图片已保存到 {} 文件", output_filename);
        println!("图片类型: {}", mime_type);
        println!("图片大小: {} bytes", image_size);
    } else {
        println!("未在响应文本中找到图片数据");
    }

    Ok(())
}