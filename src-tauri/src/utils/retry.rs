use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

/// 指数退避重试工具。
/// 仅对可重试错误（超时、5xx、连接错误）进行重试，4xx 等客户端错误直接返回。
///
/// # Arguments
/// * `max_retries` - 最大重试次数（不含首次请求，总共最多执行 max_retries + 1 次）
/// * `operation` - 异步操作闭包
pub async fn retry_with_backoff<F, Fut, T>(
    max_retries: u32,
    operation: F,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_err = None;

    for attempt in 0..=max_retries {
        match operation().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                let err_msg = e.to_string().to_lowercase();
                // 判断是否为可重试错误（超时/5xx/连接错误）
                let is_retryable = err_msg.contains("timeout")
                    || err_msg.contains("timed out")
                    || err_msg.contains("connection")
                    || err_msg.contains("500")
                    || err_msg.contains("502")
                    || err_msg.contains("503")
                    || err_msg.contains("504")
                    || err_msg.contains("server error")
                    || err_msg.contains("broken pipe")
                    || err_msg.contains("reset by peer");

                if !is_retryable || attempt == max_retries {
                    return Err(e);
                }

                last_err = Some(e);
                // 指数退避: 1s, 2s, 4s
                let delay = Duration::from_secs(1 << attempt);
                log::warn!(
                    "AI API 请求失败（第 {} 次），{}s 后重试: {}",
                    attempt + 1,
                    delay.as_secs(),
                    last_err.as_ref().map(|e| e.to_string()).unwrap_or_default()
                );
                sleep(delay).await;
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("retry exhausted")))
}
