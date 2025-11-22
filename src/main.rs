use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::env;
use base64::{Engine as _, engine::general_purpose};
use dotenv::dotenv;

#[derive(Serialize)]
struct RequestBody {
    model: String,
    messages: Vec<Message>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    #[serde(rename = "generation_config")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Serialize)]
struct GenerationConfig {
    temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
    #[serde(rename = "responseModalities")]
    response_modalities: Vec<String>,
    #[serde(rename = "topP")]
    top_p: f32,
    #[serde(rename = "imageConfig")]
    image_config: ImageConfig,
}

#[derive(Serialize)]
struct ImageConfig {
    #[serde(rename = "aspectRatio")]
    aspect_ratio: String,
    #[serde(rename = "imageSize")]
    image_size: String,
    #[serde(rename = "imageOutputOptions")]
    image_output_options: ImageOutputOptions,
    #[serde(rename = "personGeneration")]
    person_generation: String,
}

#[derive(Serialize)]
struct ImageOutputOptions {
    #[serde(rename = "mimeType")]
    mime_type: String,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
    image_url: Option<ImageUrl>,
}

#[derive(Serialize)]
struct ImageUrl {
    url: String,
}

#[derive(Deserialize)]
struct Response {
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载 .env 文件
    dotenv().ok();

    let client = Client::new();

    // 从环境变量读取配置
    let api_url = env::var("API_URL")
        .unwrap_or_else(|_| "http://ai.burncloud.com/v1/chat/completions".to_string());
    let api_key = env::var("API_KEY")
        .expect("API_KEY must be set in .env file or environment variables");
    let image_path = env::var("IMAGE_PATH")
        .unwrap_or_else(|_| "test.png".to_string());

    // 检查图片文件是否存在
    if !std::path::Path::new(&image_path).exists() {
        println!("错误: 找不到图片文件 '{}'", image_path);
        println!("请将图片文件放在项目根目录，或修改 image_path 变量指向正确的图片路径");
        return Ok(());
    }

    let image_data = fs::read(image_path)?;
    let base64_image = general_purpose::STANDARD.encode(&image_data);
    let image_url = format!("data:image/jpeg;base64,{}", base64_image);

    let request_body = RequestBody {
        model: "gemini-3-pro-image-preview".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: vec![
                    Content {
                        content_type: "text".to_string(),
                        text: Some("a cat".to_string()),
                        image_url: None,
                    },
                    Content {
                        content_type: "image_url".to_string(),
                        text: None,
                        image_url: Some(ImageUrl { url: image_url }),
                    },
                ],
            }
        ],
        max_tokens: Some(32768), // 增加到最大支持
        temperature: Some(0.7),
        generation_config: Some(GenerationConfig {
            temperature: 1.0,
            max_output_tokens: 32768,
            response_modalities: vec!["TEXT".to_string(), "IMAGE".to_string()],
            top_p: 0.95,
            image_config: ImageConfig {
                aspect_ratio: "1:1".to_string(),
                image_size: "4K".to_string(),
                image_output_options: ImageOutputOptions {
                    mime_type: "image/png".to_string(),
                },
                person_generation: "ALLOW_ALL".to_string(),
            },
        }),
    };

    println!("发送请求到 Gemini 3 Pro Image Preview...");

    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if response.status().is_success() {
        let result: Response = response.json().await?;

        if let Some(choice) = result.choices.first() {
            let content = &choice.message.content;

            // 检查是否是base64图片数据
            if content.starts_with("![image](data:image/png;base64,") || content.starts_with("data:image/") {
                // 提取base64数据部分（去掉data:image/...;base64,前缀）
                if let Some(comma_pos) = content.find(',') {
                    let mut base64_data = &content[comma_pos + 1..];

                    // 如果是Markdown格式，需要去掉末尾的右括号
                    if content.starts_with("![image](") {
                        if let Some(paren_pos) = base64_data.find(')') {
                            base64_data = &base64_data[..paren_pos];
                        }
                    }

                    // 解码base64数据
                    match general_purpose::STANDARD.decode(base64_data) {
                        Ok(image_bytes) => {
                            // 根据图片类型确定文件扩展名
                            let file_extension = if content.contains("png") {
                                "png"
                            } else if content.contains("jpeg") || content.contains("jpg") {
                                "jpg"
                            } else if content.contains("gif") {
                                "gif"
                            } else if content.contains("webp") {
                                "webp"
                            } else {
                                "png" // 默认为png
                            };

                            let output_filename = format!("response.{}", file_extension);
                            fs::write(&output_filename, image_bytes)?;
                            println!("AI回复的图片已保存到 {} 文件", output_filename);
                        }
                        Err(e) => {
                            println!("base64解码失败: {}", e);
                            // 如果解码失败，将原始内容保存为文本文件
                            fs::write("response.txt", content)?;
                            println!("原始内容已保存到 response.txt 文件");
                        }
                    }
                } else {
                    fs::write("response.txt", content)?;
                    println!("AI回复已保存到 response.txt 文件");
                }
            } else {
                // 如果不是图片数据，保存为文本文件
                fs::write("response.txt", content)?;
                println!("AI回复已保存到 response.txt 文件");
            }
        }

        println!("Token使用情况:");
        println!("  输入: {}", result.usage.prompt_tokens);
        println!("  输出: {}", result.usage.completion_tokens);
        println!("  总计: {}", result.usage.total_tokens);
    } else {
        let status = response.status();
        let headers = response.headers().clone();
        let error_text = response.text().await?;
        println!("请求失败 - 状态码: {}", status);
        println!("错误内容: {}", error_text);

        // 如果错误内容为空，显示响应头信息
        if error_text.is_empty() {
            println!("响应头: {:?}", headers);
        }

        // 将错误信息保存到文件
        fs::write("error.txt", format!("Status: {}\nHeaders: {:?}\nError: {}", status, headers, error_text))?;
        println!("错误信息已保存到 error.txt 文件");
    }

    Ok(())
}