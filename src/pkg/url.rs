#[derive(Debug, Clone, PartialEq)]
pub struct UrlBuilder {
    scheme: Option<String>,
    username: Option<String>,
    password: Option<String>,
    host: String,
    port: Option<u16>,
    path: Option<String>,
    query: Option<String>,
    fragment: Option<String>,
}

impl UrlBuilder {
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            scheme: None,
            username: None,
            password: None,
            host: host.into(),
            port: None,
            path: None,
            query: None,
            fragment: None,
        }
    }

    pub fn scheme(mut self, scheme: impl Into<String>) -> Self {
        self.scheme = Some(scheme.into());
        self
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn fragment(mut self, fragment: impl Into<String>) -> Self {
        self.fragment = Some(fragment.into());
        self
    }

    pub fn build(self) -> String {
        let mut url = String::new();

        // Scheme
        if let Some(scheme) = &self.scheme {
            url.push_str(scheme);
            url.push_str("://");
        }

        // Username and password
        if let Some(username) = &self.username {
            url.push_str(username);
            if let Some(password) = &self.password {
                url.push(':');
                url.push_str(password);
            }
            url.push('@');
        }

        // Host
        url.push_str(&self.host);

        // Port
        if let Some(port) = self.port {
            url.push(':');
            url.push_str(&port.to_string());
        }

        // Path
        if let Some(path) = &self.path {
            if !path.starts_with('/') {
                url.push('/');
            }
            url.push_str(path);
        }

        // Query
        if let Some(query) = &self.query {
            url.push('?');
            url.push_str(query);
        }

        // Fragment
        if let Some(fragment) = &self.fragment {
            url.push('#');
            url.push_str(fragment);
        }

        url
    }
}

impl std::fmt::Display for UrlBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.clone().build())
    }
}

/// Encode URL components to be URL-safe
pub fn url_encode(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

/// Decode URL-encoded components
pub fn url_decode(input: &str) -> Result<String, anyhow::Error> {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '%' => {
                let hex_chars: String = chars.by_ref().take(2).collect();
                if hex_chars.len() == 2 {
                    let byte = u8::from_str_radix(&hex_chars, 16)?;
                    result.push(byte as char);
                } else {
                    return Err(anyhow::anyhow!("Invalid URL encoding"));
                }
            }
            '+' => result.push(' '),
            _ => result.push(c),
        }
    }

    Ok(result)
}
