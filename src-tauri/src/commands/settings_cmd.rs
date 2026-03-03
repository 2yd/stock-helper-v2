use tauri::{AppHandle, Manager, State};
use crate::AppState;
use crate::models::settings::AppSettings;
use crate::models::ai::AIConfig;
use crate::models::strategy::StrategyConfig;
use crate::services::ai_service::AIService;

#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    state.db.load_settings().map_err(|e| {
        log::error!("[settings_cmd] get_settings failed: {}", e);
        e.to_string()
    })
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    log::info!("[settings_cmd] save_settings");
    state.db.save_settings(&settings).map_err(|e| {
        log::error!("[settings_cmd] save_settings failed: {}", e);
        e.to_string()
    })
}

#[tauri::command]
pub async fn add_ai_config(
    state: State<'_, AppState>,
    config: AIConfig,
) -> Result<AppSettings, String> {
    log::info!("[settings_cmd] add_ai_config model={}", config.model_name);
    let mut settings = state.db.load_settings().map_err(|e| e.to_string())?;
    settings.ai_configs.push(config);
    if settings.active_ai_config_id.is_none() {
        settings.active_ai_config_id = settings.ai_configs.first().map(|c| c.id.clone());
    }
    state.db.save_settings(&settings).map_err(|e| e.to_string())?;
    Ok(settings)
}

#[tauri::command]
pub async fn remove_ai_config(
    state: State<'_, AppState>,
    config_id: String,
) -> Result<AppSettings, String> {
    log::info!("[settings_cmd] remove_ai_config id={}", config_id);
    let mut settings = state.db.load_settings().map_err(|e| e.to_string())?;
    settings.ai_configs.retain(|c| c.id != config_id);
    if settings.active_ai_config_id.as_deref() == Some(&config_id) {
        settings.active_ai_config_id = settings.ai_configs.first().map(|c| c.id.clone());
    }
    state.db.save_settings(&settings).map_err(|e| e.to_string())?;
    Ok(settings)
}

#[tauri::command]
pub async fn update_ai_config(
    state: State<'_, AppState>,
    config: AIConfig,
) -> Result<AppSettings, String> {
    log::info!("[settings_cmd] update_ai_config model={}", config.model_name);
    let mut settings = state.db.load_settings().map_err(|e| e.to_string())?;
    if let Some(existing) = settings.ai_configs.iter_mut().find(|c| c.id == config.id) {
        *existing = config;
    }
    state.db.save_settings(&settings).map_err(|e| e.to_string())?;
    Ok(settings)
}

#[tauri::command]
pub async fn set_active_ai_config(
    state: State<'_, AppState>,
    config_id: String,
) -> Result<AppSettings, String> {
    log::info!("[settings_cmd] set_active_ai_config id={}", config_id);
    let mut settings = state.db.load_settings().map_err(|e| e.to_string())?;
    settings.active_ai_config_id = Some(config_id);
    state.db.save_settings(&settings).map_err(|e| e.to_string())?;
    Ok(settings)
}

#[tauri::command]
pub async fn update_strategy_config(
    state: State<'_, AppState>,
    strategy: StrategyConfig,
) -> Result<AppSettings, String> {
    log::info!("[settings_cmd] update_strategy_config id={}", strategy.id);
    let mut settings = state.db.load_settings().map_err(|e| e.to_string())?;
    if let Some(existing) = settings.strategies.iter_mut().find(|s| s.id == strategy.id) {
        *existing = strategy;
    } else {
        settings.strategies.push(strategy);
    }
    state.db.save_settings(&settings).map_err(|e| e.to_string())?;
    Ok(settings)
}

/// 测试 AI 模型配置是否可用
#[tauri::command]
pub async fn test_ai_config(
    config: AIConfig,
) -> Result<String, String> {
    log::info!("[settings_cmd] test_ai_config model={}", config.model_name);
    AIService::test_ai_connection(&config).await.map_err(|e| {
        log::error!("[settings_cmd] test_ai_config failed: {}", e);
        e.to_string()
    })
}

/// 检查版本更新：通过 GitHub Releases API 获取最新版本，与当前版本对比
#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<serde_json::Value>, String> {
    log::info!("[settings_cmd] check_update");

    // 从 tauri.conf.json 读取当前版本
    let current_version = app.config().version.clone().unwrap_or_default();
    log::info!("[settings_cmd] current_version={}", current_version);

    // 调用 GitHub API 获取最新 release
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let resp = client
        .get("https://api.github.com/repos/2yd/stock-helper-v2/releases/latest")
        .header("User-Agent", "StockHelper")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("请求 GitHub API 失败: {}", e))?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        log::info!("[settings_cmd] check_update: no releases found");
        return Ok(None);
    }

    if !resp.status().is_success() {
        let status = resp.status();
        return Err(format!("GitHub API 返回错误: {}", status));
    }

    let release: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析 GitHub API 响应失败: {}", e))?;

    let tag_name = release["tag_name"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches('v')
        .to_string();

    if tag_name.is_empty() {
        return Ok(None);
    }

    // 比较版本号
    let is_newer = compare_versions(&tag_name, &current_version);
    if !is_newer {
        log::info!(
            "[settings_cmd] check_update: current={} latest={}, no update",
            current_version,
            tag_name
        );
        return Ok(None);
    }

    log::info!(
        "[settings_cmd] check_update: new version available {} -> {}",
        current_version,
        tag_name
    );

    // 构建更新信息
    let update_info = serde_json::json!({
        "version": tag_name,
        "current_version": current_version,
        "body": release["body"].as_str().unwrap_or(""),
        "published_at": release["published_at"].as_str().unwrap_or(""),
        "html_url": release["html_url"].as_str().unwrap_or(""),
    });

    Ok(Some(update_info))
}

/// 简单的语义化版本对比：latest > current 返回 true
fn compare_versions(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> {
        v.split('.')
            .filter_map(|s| s.parse::<u64>().ok())
            .collect()
    };
    let l = parse(latest);
    let c = parse(current);
    for i in 0..l.len().max(c.len()) {
        let lv = l.get(i).copied().unwrap_or(0);
        let cv = c.get(i).copied().unwrap_or(0);
        if lv > cv {
            return true;
        }
        if lv < cv {
            return false;
        }
    }
    false
}

/// 导出日志：将 app_log_dir 下所有日志文件打包为 zip，通过系统对话框让用户选择保存位置
#[tauri::command]
pub async fn export_logs(app: AppHandle) -> Result<String, String> {
    use std::io::{Read, Write};
    use tauri_plugin_dialog::DialogExt;
    use zip::write::SimpleFileOptions;

    log::info!("[settings_cmd] export_logs");

    // 获取日志目录
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|e| format!("获取日志目录失败: {}", e))?;

    if !log_dir.exists() {
        return Err("日志目录不存在".into());
    }

    // 收集所有日志文件
    let entries = std::fs::read_dir(&log_dir)
        .map_err(|e| format!("读取日志目录失败: {}", e))?;

    let log_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.is_file()
                && path
                    .extension()
                    .map(|ext| ext == "log")
                    .unwrap_or(false)
        })
        .collect();

    if log_files.is_empty() {
        return Err("没有找到日志文件".into());
    }

    log::info!(
        "[settings_cmd] export_logs found {} log files",
        log_files.len()
    );

    // 在内存中创建 zip
    let buf = std::io::Cursor::new(Vec::new());
    let mut zip_writer = zip::ZipWriter::new(buf);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for entry in &log_files {
        let path = entry.path();
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let mut file = std::fs::File::open(&path)
            .map_err(|e| format!("打开日志文件 {} 失败: {}", file_name, e))?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .map_err(|e| format!("读取日志文件 {} 失败: {}", file_name, e))?;

        zip_writer
            .start_file(&file_name, options)
            .map_err(|e| format!("写入 zip 失败: {}", e))?;
        zip_writer
            .write_all(&contents)
            .map_err(|e| format!("写入 zip 内容失败: {}", e))?;
    }

    let cursor = zip_writer
        .finish()
        .map_err(|e| format!("完成 zip 文件失败: {}", e))?;
    let zip_data = cursor.into_inner();

    // 通过系统对话框让用户选择保存位置
    let file_path = app
        .dialog()
        .file()
        .set_title("导出日志")
        .set_file_name("stock-helper-logs.zip")
        .add_filter("ZIP 文件", &["zip"])
        .blocking_save_file();

    match file_path {
        Some(path) => {
            std::fs::write(path.as_path().unwrap(), &zip_data)
                .map_err(|e| format!("保存文件失败: {}", e))?;
            log::info!("[settings_cmd] export_logs saved to {:?}", path);
            Ok(format!("日志已导出（{} 个文件）", log_files.len()))
        }
        None => {
            log::info!("[settings_cmd] export_logs cancelled by user");
            Err("用户取消了导出".into())
        }
    }
}
