//! Email delivery for verification and notification messages.

use crate::server::state::SmtpConfig;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};

/// Send an email verification link to the given address.
///
/// If `smtp` is `None` the email is silently skipped (logged as a warning).
///
/// # Errors
///
/// Returns an error string if the SMTP send fails.
pub async fn send_verification_email(
    smtp: Option<&SmtpConfig>,
    to_email: &str,
    token: &str,
    base_url: &str,
) -> Result<(), String> {
    let Some(config) = smtp else {
        tracing::warn!(
            to_email,
            "SMTP not configured — skipping verification email"
        );
        return Ok(());
    };

    let from: Mailbox = config
        .from
        .parse()
        .map_err(|e| format!("invalid EMAIL_FROM address: {e}"))?;
    let to: Mailbox = to_email
        .parse()
        .map_err(|e| format!("invalid recipient address: {e}"))?;

    let verify_url = format!("{base_url}/auth/verify-email?token={token}");

    let email = lettre::Message::builder()
        .from(from)
        .to(to)
        .subject("Verify your email address")
        .header(ContentType::TEXT_PLAIN)
        .body(format!(
            "Please verify your email address by clicking the link below:\n\n\
             {verify_url}\n\n\
             This link will expire in 24 hours.\n\n\
             If you did not request this, you can safely ignore this email."
        ))
        .map_err(|e| format!("failed to build email: {e}"))?;

    let creds = Credentials::new(config.username.clone(), config.password.clone());

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
        .map_err(|e| format!("SMTP relay error: {e}"))?
        .port(config.port)
        .credentials(creds)
        .build();

    mailer
        .send(email)
        .await
        .map_err(|e| format!("SMTP send error: {e}"))?;

    tracing::info!(to_email, "verification email sent");
    Ok(())
}
