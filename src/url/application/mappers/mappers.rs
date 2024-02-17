use crate::url::application::dtos::url_dto::URLInfoDto;
use crate::url::domain::models::schema::URL;

// Funció per mapejar URL a URLInfoDto
pub fn map_url_to_dto(url: &URL) -> URLInfoDto {
    URLInfoDto {
        target_url: url.target_url.clone(),
        clicks: url.clicks,
        is_active: url.is_active,
        url: format!("/{}", url.key),
        admin_url: format!("/admin/{}", url.secret_key),
    }
}
