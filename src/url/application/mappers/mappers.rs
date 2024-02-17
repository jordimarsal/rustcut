use crate::config::env::AppConfig;
use crate::url::application::dtos::url_dto::URLInfoDto;
use crate::url::domain::models::schema::URL;

// Funció per mapejar URL a URLInfoDto
pub fn map_url_to_dto(url: &URL, config: AppConfig) -> URLInfoDto {
    let base_url = format!(
        "{protocol}://{base_url}:{server_port}",
        protocol = config.protocol,
        base_url = config.base_url,
        server_port = config.server_port
    );
    URLInfoDto {
        target_url: url.target_url.clone(),
        clicks: url.clicks,
        is_active: url.is_active,
        url: format!("{base_url}/{}", url.key),
        admin_url: format!("{base_url}/admin/{}", url.secret_key),
    }
}
