#[cfg(feature = "desktop")]
pub mod account;
#[cfg(any(feature = "desktop", feature = "cli"))]
pub mod account_index_repair;
#[cfg(feature = "desktop")]
pub mod announcement;
#[cfg(feature = "desktop")]
pub mod antigravity_switch_history;
#[cfg(any(feature = "desktop", feature = "cli"))]
pub mod atomic_write;
#[cfg(feature = "desktop")]
pub mod client_version;
#[cfg(feature = "desktop")]
pub mod codebuddy_account;
#[cfg(feature = "desktop")]
pub mod codebuddy_cn_account;
#[cfg(feature = "desktop")]
pub mod codebuddy_cn_instance;
#[cfg(feature = "desktop")]
pub mod codebuddy_cn_oauth;
#[cfg(feature = "desktop")]
pub mod codebuddy_instance;
#[cfg(feature = "desktop")]
pub mod codebuddy_oauth;
#[cfg(feature = "desktop")]
pub mod codex_account;
#[cfg(feature = "desktop")]
pub mod codex_instance;
#[cfg(feature = "desktop")]
pub mod codex_oauth;
#[cfg(feature = "desktop")]
pub mod codex_quota;
// pub mod codex_session_manager;
// pub mod codex_session_visibility;
// pub mod codex_thread_sync;
// pub mod codex_wakeup;
// pub mod codex_wakeup_scheduler;
#[cfg(feature = "cli")]
pub mod cli_account;
#[cfg(any(feature = "desktop", feature = "cli"))]
pub mod config;
#[cfg(any(feature = "desktop", feature = "cli"))]
pub mod cursor_account;
#[cfg(feature = "desktop")]
pub mod cursor_instance;
#[cfg(feature = "desktop")]
pub mod cursor_oauth;
#[cfg(feature = "desktop")]
pub mod db;
// pub mod external_import;
// pub mod floating_card_window;
#[cfg(any(feature = "desktop", feature = "cli"))]
pub mod github_copilot_account;
#[cfg(feature = "desktop")]
pub mod github_copilot_instance;
#[cfg(any(feature = "desktop", feature = "cli"))]
pub mod github_copilot_oauth;
// pub mod group_settings;
#[cfg(any(feature = "desktop", feature = "cli"))]
pub mod i18n;
#[cfg(feature = "desktop")]
pub mod import;
#[cfg(feature = "desktop")]
pub mod instance;
#[cfg(feature = "desktop")]
pub mod instance_store;
#[cfg(feature = "desktop")]
pub mod kiro_account;
#[cfg(feature = "desktop")]
pub mod kiro_instance;
#[cfg(feature = "desktop")]
pub mod kiro_oauth;
// pub mod linux_updater;
#[cfg(any(feature = "desktop", feature = "cli"))]
pub mod logger;
// pub mod macos_native_menu;
#[cfg(feature = "desktop")]
pub mod oauth;
#[cfg(feature = "desktop")]
pub mod oauth_pending_state;
// pub mod oauth_server;
// pub mod openclaw_auth;
// pub mod opencode_auth;
#[cfg(feature = "desktop")]
pub mod process;
#[cfg(feature = "desktop")]
pub mod qoder_account;
#[cfg(feature = "desktop")]
pub mod qoder_instance;
#[cfg(feature = "desktop")]
pub mod qoder_oauth;
#[cfg(feature = "desktop")]
pub mod quota;
#[cfg(feature = "desktop")]
pub mod quota_cache;
// pub mod sync_settings;
#[cfg(feature = "desktop")]
pub mod trae_account;
#[cfg(feature = "desktop")]
pub mod trae_instance;
#[cfg(feature = "desktop")]
pub mod trae_oauth;
// pub mod tray;
// pub mod tray_layout;
// pub mod update_checker;
#[cfg(feature = "desktop")]
pub mod vscode_inject;
#[cfg(feature = "desktop")]
pub mod vscode_paths;
// pub mod wakeup;
#[cfg(feature = "desktop")]
pub mod wakeup_gateway;
// pub mod wakeup_history;
// pub mod wakeup_scheduler;
// pub mod wakeup_verification;
// pub mod web_report;
#[cfg(feature = "desktop")]
pub mod websocket;
#[cfg(feature = "desktop")]
pub mod windsurf_account;
#[cfg(feature = "desktop")]
pub mod windsurf_devin_oauth;
#[cfg(feature = "desktop")]
pub mod windsurf_instance;
#[cfg(feature = "desktop")]
pub mod windsurf_oauth;
#[cfg(feature = "desktop")]
pub mod workbuddy_account;
#[cfg(feature = "desktop")]
pub mod workbuddy_instance;
#[cfg(feature = "desktop")]
pub mod workbuddy_oauth;
#[cfg(feature = "desktop")]
pub mod zed_account;
#[cfg(feature = "desktop")]
pub mod zed_instance;
#[cfg(feature = "desktop")]
pub mod zed_oauth;

// 重新导出常用函数
#[cfg(feature = "desktop")]
pub use account::*;
