use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct QuotaAlertPayload {
    pub platform: String,
    pub current_account_id: String,
    pub current_email: String,
    pub threshold: i32,
    pub threshold_display: Option<String>,
    pub lowest_percentage: i32,
    pub low_models: Vec<String>,
    pub recommended_account_id: Option<String>,
    pub recommended_email: Option<String>,
    pub triggered_at: i64,
}

pub fn get_data_dir() -> Result<PathBuf, String> {
    let data_dir = crate::modules::config::get_data_dir()?;
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).map_err(|err| format!("创建数据目录失败: {}", err))?;
    }
    Ok(data_dir)
}

pub fn dispatch_quota_alert(payload: &QuotaAlertPayload) {
    crate::modules::logger::log_warn(&format!(
        "[QuotaAlert] platform={}, current_id={}, threshold={}%, lowest={}%",
        payload.platform, payload.current_account_id, payload.threshold, payload.lowest_percentage
    ));
}
