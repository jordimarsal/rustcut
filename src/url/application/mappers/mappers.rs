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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::env::AppConfig;

    #[test]
    fn map_url_to_dto_builds_correct_urls() {
        let url = URL { key: "K".into(), secret_key: "S".into(), target_url: "http://t".into(), is_active: true, clicks: 3, user_id: 1 };
        let cfg = AppConfig { base_url: "localhost".into(), server_port: "8080".into(), protocol: "http".into() };
        let dto = map_url_to_dto(&url, cfg);
        assert!(dto.url.contains("localhost:8080/K"));
        assert!(dto.admin_url.contains("localhost:8080/admin/S"));
        assert_eq!(dto.clicks, 3);
    }
}
