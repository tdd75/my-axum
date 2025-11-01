use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

#[derive(Clone, Debug)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

impl SmtpConfig {
    pub fn new(host: String, port: u16, username: String, password: String, use_tls: bool) -> Self {
        Self {
            host,
            port,
            username,
            password,
            use_tls,
        }
    }

    pub fn gmail(username: String, password: String) -> Self {
        Self::new("smtp.gmail.com".to_string(), 587, username, password, true)
    }

    pub fn outlook(username: String, password: String) -> Self {
        Self::new(
            "smtp-mail.outlook.com".to_string(),
            587,
            username,
            password,
            true,
        )
    }

    pub fn localhost(port: u16) -> Self {
        Self::new(
            "localhost".to_string(),
            port,
            "test@localhost".to_string(),
            "test".to_string(),
            false,
        )
    }
}

#[derive(Clone, Debug)]
pub struct SmtpClient {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    sender: String,
}

impl SmtpClient {
    pub fn new(config: SmtpConfig) -> anyhow::Result<Self> {
        let creds = Credentials::new(config.username.clone(), config.password);

        let transport = if config.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
                .map_err(|e| {
                    anyhow::anyhow!("Failed to create TLS transport for {}: {}", config.host, e)
                })?
                .port(config.port)
                .credentials(creds)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
                .port(config.port)
                .credentials(creds)
                .build()
        };

        Ok(Self {
            transport,
            sender: config.username,
        })
    }

    pub fn from_params(
        host: String,
        port: u16,
        username: String,
        password: String,
        use_tls: bool,
    ) -> anyhow::Result<Self> {
        let config = SmtpConfig::new(host, port, username, password, use_tls);
        Self::new(config)
    }

    pub async fn send_mail(
        &self,
        to: &str,
        subject: &str,
        body: String,
        content_type: ContentType,
    ) -> anyhow::Result<()> {
        let email = Message::builder()
            .from(
                self.sender
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid sender email {}: {}", self.sender, e))?,
            )
            .to(to
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid recipient email {}: {}", to, e))?)
            .subject(subject)
            .header(content_type)
            .body(body)
            .map_err(|e| anyhow::anyhow!("Failed to build email message: {}", e))?;

        self.transport
            .send(email)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send email: {}", e))?;

        Ok(())
    }

    pub async fn send_text_mail(
        &self,
        to: &str,
        subject: &str,
        body: String,
    ) -> anyhow::Result<()> {
        self.send_mail(to, subject, body, ContentType::TEXT_PLAIN)
            .await
    }

    pub async fn send_html_mail(
        &self,
        to: &str,
        subject: &str,
        body: String,
    ) -> anyhow::Result<()> {
        self.send_mail(to, subject, body, ContentType::TEXT_HTML)
            .await
    }

    pub async fn send_multipart_mail(
        &self,
        to: &str,
        subject: &str,
        text_body: String,
        html_body: String,
    ) -> anyhow::Result<()> {
        use lettre::{message::MultiPart, message::SinglePart};

        let email = Message::builder()
            .from(
                self.sender
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid sender email {}: {}", self.sender, e))?,
            )
            .to(to
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid recipient email {}: {}", to, e))?)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text_body),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html_body),
                    ),
            )
            .map_err(|e| anyhow::anyhow!("Failed to build multipart email: {}", e))?;

        self.transport
            .send(email)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send multipart email: {}", e))?;

        Ok(())
    }

    pub async fn test_connection(&self) -> anyhow::Result<()> {
        // Test connection by sending NOOP command
        self.transport
            .test_connection()
            .await
            .map_err(|e| anyhow::anyhow!("SMTP connection test failed: {}", e))?;

        Ok(())
    }
}
