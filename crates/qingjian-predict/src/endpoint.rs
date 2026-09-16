//! 接口地址的判断与规整：哪算本机 / 局域网，要不要补 `/v1`。
//!
//! LM Studio、Ollama、llama.cpp 这类自建服务的 OpenAI 兼容接口与云端服务商有两个实际差别，集中在这里判一次：
//! 一是不认 OpenAI 的 `response_format: json_object`——LM Studio 直接回 400
//! `'response_format.type' must be 'json_schema' or 'text'`；二是不要密钥，填不填都能用。
//! 它们的接口都在 `/v1` 下，用户只写 `http://127.0.0.1:12345` 时也要能用。

/// 地址指向本机或局域网（`localhost` / 回环 / 私有网段 / `.local`）：自建服务多半在这里。
pub(crate) fn is_local(base_url: &str) -> bool {
    let Some(host) = host(base_url) else {
        return false;
    };
    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".local") {
        return true;
    }
    match host.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(ip)) => ip.is_loopback() || ip.is_private() || ip.is_link_local(),
        Ok(std::net::IpAddr::V6(ip)) => ip.is_loopback(),
        Err(_) => false,
    }
}

/// 要不要发 `response_format: json_object`：自建服务多半不认，不发，靠提示词里的 JSON 约定。
pub(crate) fn wants_json_object(base_url: &str) -> bool {
    !is_local(base_url)
}

/// 规整接口地址：路径为空（`http://127.0.0.1:12345` 或末尾一个 `/`）时补上 `/v1`，其余原样。
/// OpenAI 兼容接口约定在 `/v1` 下，漏了会得到一句 `Unexpected endpoint or method.`；DeepSeek 这类
/// 服务商的 base_url 带不带 `/v1` 都行，补上不影响。
pub(crate) fn normalize(base_url: &str) -> String {
    let trimmed = base_url.trim().trim_end_matches('/').to_owned();
    let Ok(url) = reqwest::Url::parse(&trimmed) else {
        return trimmed;
    };
    if url.path() == "/" {
        return format!("{trimmed}/v1");
    }
    trimmed
}

/// 地址里的主机名，小写、去掉 IPv6 的方括号；地址解析不了返回 `None`。
fn host(base_url: &str) -> Option<String> {
    let url = reqwest::Url::parse(base_url.trim()).ok()?;
    let host = url.host_str()?.trim_matches(|c| c == '[' || c == ']');
    Some(host.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::{is_local, normalize, wants_json_object};

    #[test]
    fn loopback_and_lan_are_local() {
        assert!(is_local("http://127.0.0.1:12345/v1"));
        assert!(is_local("http://localhost:11434"));
        assert!(is_local("http://[::1]:8080/v1"));
        assert!(is_local("http://192.168.1.9:1234/v1"));
        assert!(is_local("http://10.0.0.5:8000/v1"));
        assert!(is_local("http://172.16.3.4:8000/v1"));
        assert!(is_local("http://macbook.local:1234/v1"));
        assert!(!is_local("https://api.deepseek.com"));
        assert!(!is_local("https://opencode.ai/zen/go/v1"));
        assert!(!is_local("http://172.32.0.1:8000/v1"));
        assert!(!is_local("不是地址"));
    }

    #[test]
    fn local_services_skip_json_object() {
        assert!(!wants_json_object("http://127.0.0.1:12345/v1"));
        assert!(wants_json_object("https://api.deepseek.com"));
    }

    #[test]
    fn bare_urls_get_v1_appended() {
        assert_eq!(
            normalize("http://127.0.0.1:12345"),
            "http://127.0.0.1:12345/v1"
        );
        assert_eq!(
            normalize("http://127.0.0.1:12345/"),
            "http://127.0.0.1:12345/v1"
        );
        assert_eq!(
            normalize(" https://api.deepseek.com "),
            "https://api.deepseek.com/v1"
        );
        assert_eq!(
            normalize("https://api.openai.com/v1/"),
            "https://api.openai.com/v1"
        );
        assert_eq!(
            normalize("https://opencode.ai/zen/go/v1"),
            "https://opencode.ai/zen/go/v1"
        );
        assert_eq!(normalize("不是地址"), "不是地址");
    }
}
