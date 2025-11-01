use pkg::smtp::{SmtpClient, SmtpConfig};
use std::time::Duration;

#[tokio::test]
async fn test_send_text_mail_rejects_invalid_sender_before_network() {
    let client = SmtpClient::new(SmtpConfig::new(
        "localhost".to_string(),
        1025,
        "not-an-email".to_string(),
        "password".to_string(),
        false,
    ))
    .unwrap();

    let error = client
        .send_text_mail("user@example.com", "Subject", "body".to_string())
        .await
        .unwrap_err();

    assert!(error.to_string().contains("Invalid sender email"));
}

#[tokio::test]
async fn test_send_html_mail_rejects_invalid_sender_before_network() {
    let client = SmtpClient::new(SmtpConfig::new(
        "localhost".to_string(),
        1025,
        "not-an-email".to_string(),
        "password".to_string(),
        false,
    ))
    .unwrap();

    let error = client
        .send_html_mail("user@example.com", "Subject", "<p>body</p>".to_string())
        .await
        .unwrap_err();

    assert!(error.to_string().contains("Invalid sender email"));
}

#[tokio::test]
async fn test_send_html_mail_rejects_invalid_recipient_before_network() {
    let client = SmtpClient::new(SmtpConfig::localhost(1025)).unwrap();

    let error = client
        .send_html_mail("not-an-email", "Subject", "<p>body</p>".to_string())
        .await
        .unwrap_err();

    assert!(error.to_string().contains("Invalid recipient email"));
}

#[tokio::test]
async fn test_send_multipart_mail_rejects_invalid_recipient_before_network() {
    let client = SmtpClient::new(SmtpConfig::localhost(1025)).unwrap();

    let error = client
        .send_multipart_mail(
            "not-an-email",
            "Subject",
            "plain text".to_string(),
            "<p>html</p>".to_string(),
        )
        .await
        .unwrap_err();

    assert!(error.to_string().contains("Invalid recipient email"));
}

#[tokio::test]
async fn test_send_multipart_mail_rejects_invalid_sender_before_network() {
    let client = SmtpClient::new(SmtpConfig::new(
        "localhost".to_string(),
        1025,
        "not-an-email".to_string(),
        "password".to_string(),
        false,
    ))
    .unwrap();

    let error = client
        .send_multipart_mail(
            "user@example.com",
            "Subject",
            "plain text".to_string(),
            "<p>html</p>".to_string(),
        )
        .await
        .unwrap_err();

    assert!(error.to_string().contains("Invalid sender email"));
}

#[tokio::test]
async fn test_test_connection_surfaces_transport_errors() {
    let client = SmtpClient::new(SmtpConfig::localhost(1)).unwrap();

    let error = tokio::time::timeout(Duration::from_secs(5), client.test_connection())
        .await
        .expect("SMTP connection test should fail quickly")
        .unwrap_err();

    assert!(error.to_string().contains("SMTP connection test failed"));
}
