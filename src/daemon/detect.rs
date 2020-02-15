use std::net::IpAddr;
use std::str::FromStr;
use std::process;

use crate::{config, daemon};
use crate::daemon::RefreshErrorKind;

/// IP detection via a command / shell script
pub(crate) fn cmd(config: &config::DetectCmdConfig) -> daemon::RefreshResult<IpAddr> {
    // Execute the command
    log::debug!("Detecting IP using cmd {}", config.cmd);
    let output = process::Command::new(&config.cmd)
        .output()
        .map_err(|e| RefreshErrorKind::DetectError(Box::new(e)))?;

    // Parse
    let ip = std::str::from_utf8(&output.stdout)
        .map(|output| output.trim())
        .map_err(|e| RefreshErrorKind::DetectError(Box::new(e)))?;

    // Parse the string into an IP address
    IpAddr::from_str(ip)
        .map_err(|e| RefreshErrorKind::DetectError(Box::new(e)))
}

/// IP detection using a remote site
pub(crate) fn http(config: &config::DetectHttpConfig) -> daemon::RefreshResult<IpAddr> {
    // Execute the HTTP request
    log::debug!("Detecting IP using uri {}", config.uri);
    let response = reqwest::blocking::get(&config.uri)?
        .text()?;

    // Extract the IP from the response
    let ip = if let Some(regex) = &config.regex {
        // Compile the regex
        let pattern = regex::Regex::new(regex)
            .map_err(|e| RefreshErrorKind::DetectError(Box::new(e)))?;

        // Find it
        pattern.captures(&response)
            .map(|captures| captures.get(1).unwrap().as_str())
            .ok_or_else(|| RefreshErrorKind::RuntimeError(format!("Unable to find IP in {} with regex {}", response, regex)))?
    } else {
        &response
    };

    // Parse the string into an IP address
    IpAddr::from_str(ip.trim())
        .map_err(|e| RefreshErrorKind::DetectError(Box::new(e)))
}

#[cfg(test)]
mod tests {
    use crate::config::{DetectHttpConfig, DetectCmdConfig};
    use crate::util::test;

    use super::*;

    // Verifies we can detect using a script
    #[test]
    fn test_cmd() {
        // Set up the config needed to execute the test script
        let script = test::fixture_filename("detect-ip.sh");
        let config = DetectCmdConfig { cmd: script };

        // Run it
        let detected = cmd(&config).unwrap();
        assert_eq!(IpAddr::from_str("127.1.2.3").unwrap(), detected);
    }

    // Verifies a simple HTTP response with the IP address works
    #[test]
    fn test_http_basic() {
        // Set up the server to respond with a simple IP
        let _server = mockito::mock("GET", "/ip")
            .with_status(200)
            .with_body("127.1.2.3")
            .create();

        // Set up the config needed to hit the mock server
        let server_url = &mockito::server_url();
        let config = DetectHttpConfig { uri: format!("{}/ip", server_url), regex: None };

        // Run it
        let detected = http(&config).unwrap();
        assert_eq!(IpAddr::from_str("127.1.2.3").unwrap(), detected);
    }

    // Verifies we can extract the IP from a more complex response
    #[test]
    fn test_http_regex() {
        // Set up the server to respond with a simple IP
        let _server = mockito::mock("GET", "/ip")
            .with_status(200)
            .with_body("<html><body>Your IP is: 127.1.2.3</body></html>")
            .create();

        // Set up the config needed to hit the mock server
        let server_url = &mockito::server_url();
        let config = DetectHttpConfig { uri: format!("{}/ip", server_url), regex: Some(": (.*?)<".into()) };

        // Run it
        let detected = http(&config).unwrap();
        assert_eq!(IpAddr::from_str("127.1.2.3").unwrap(), detected);
    }
}
