use crate::database::{get_setting, set_setting};
use crate::error::AppResult;
use crate::models::AppSettings;

pub async fn load(db: &sqlx::SqlitePool) -> AppResult<AppSettings> {
    let mut settings = AppSettings::default();
    if let Some(v) = get_setting(db, "default_servers_dir").await? {
        settings.default_servers_dir = v;
    }
    settings.preferred_java_path = get_setting(db, "preferred_java_path").await?;
    if let Some(v) = get_setting(db, "theme").await? {
        settings.theme = v;
    }
    if let Some(v) = get_setting(db, "reduced_motion").await? {
        settings.reduced_motion = v == "true";
    }
    if let Some(v) = get_setting(db, "font_scale").await? {
        settings.font_scale = v.parse().unwrap_or(1.0);
    }
    if let Some(v) = get_setting(db, "developer_mode").await? {
        settings.developer_mode = v == "true";
    }
    if let Some(v) = get_setting(db, "backup_retention").await? {
        settings.backup_retention = v.parse().unwrap_or(10);
    }
    if let Some(v) = get_setting(db, "auto_updates").await? {
        settings.auto_updates = v == "true";
    }
    if let Some(v) = get_setting(db, "onboarded").await? {
        settings.onboarded = v == "true";
    }
    if let Some(v) = get_setting(db, "app_auto_update").await? {
        settings.app_auto_update = v == "true";
    }
    if let Some(v) = get_setting(db, "app_auto_install").await? {
        settings.app_auto_install = v == "true";
    }
    settings.skipped_app_version = get_setting(db, "skipped_app_version").await?.filter(|v| !v.is_empty());
    Ok(settings)
}

pub async fn save(db: &sqlx::SqlitePool, settings: &AppSettings) -> AppResult<()> {
    set_setting(db, "default_servers_dir", &settings.default_servers_dir).await?;
    set_setting(
        db,
        "preferred_java_path",
        settings.preferred_java_path.as_deref().unwrap_or(""),
    )
    .await?;
    set_setting(db, "theme", &settings.theme).await?;
    set_setting(db, "reduced_motion", if settings.reduced_motion { "true" } else { "false" }).await?;
    set_setting(db, "font_scale", &settings.font_scale.to_string()).await?;
    set_setting(db, "developer_mode", if settings.developer_mode { "true" } else { "false" }).await?;
    set_setting(db, "backup_retention", &settings.backup_retention.to_string()).await?;
    set_setting(db, "auto_updates", if settings.auto_updates { "true" } else { "false" }).await?;
    set_setting(db, "onboarded", if settings.onboarded { "true" } else { "false" }).await?;
    set_setting(db, "app_auto_update", if settings.app_auto_update { "true" } else { "false" }).await?;
    set_setting(db, "app_auto_install", if settings.app_auto_install { "true" } else { "false" }).await?;
    set_setting(db, "skipped_app_version", settings.skipped_app_version.as_deref().unwrap_or("")).await?;
    Ok(())
}
