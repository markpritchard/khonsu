use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) detect: DetectConfig,
    pub(crate) service: ServiceConfig,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DetectConfig {
    pub(crate) cmd: Option<DetectCmdConfig>,
    pub(crate) http: Option<DetectHttpConfig>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DetectCmdConfig {
    pub(crate) cmd: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DetectHttpConfig {
    pub(crate) uri: String,
    pub(crate) regex: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ServiceConfig {
    pub(crate) cloudflare: Option<ServiceCloudflareConfig>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ServiceCloudflareConfig {
    pub(crate) api_token: String,
    pub(crate) zone_id: String,
    pub(crate) record_type: String,
    pub(crate) name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Verify we can parse TOML configuration for a command-based detector
    #[test]
    fn parse_detect_cmd() {
        let toml_str = r#"
        [detect.cmd]
        cmd = "ssh host \"echo $SSH_CLIENT | cut -f1 -d' '\""
        [service]
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();

        assert_eq!("ssh host \"echo $SSH_CLIENT | cut -f1 -d' '\"", config.detect.cmd.unwrap().cmd);
    }

    // Verify we can parse TOML configuration for an http-based detector
    #[test]
    fn parse_detect_http() {
        let toml_str = r#"
        [detect.http]
        uri = "https://ifconfig.co/ip"
        regex = "test only"
        [service]
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();

        let detect_http = config.detect.http.unwrap();
        assert_eq!("https://ifconfig.co/ip", detect_http.uri);
        assert_eq!("test only", detect_http.regex.unwrap());
    }

    // Verify we can parse TOML configuration for a Cloudflare DNS service
    #[test]
    fn parse_service_cloudflare() {
        let toml_str = r#"
        [detect]

        [service.cloudflare]
        api_token = "test_api_token"
        zone_id = "test_zone_id"
        record_type = "test_record_type"
        name = "test_name"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();

        let cf = config.service.cloudflare.unwrap();
        assert_eq!("test_api_token", cf.api_token);
        assert_eq!("test_zone_id", cf.zone_id);
        assert_eq!("test_record_type", cf.record_type);
        assert_eq!("test_name", cf.name);
    }
}
