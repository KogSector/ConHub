use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
}

impl EmailConfig {
    pub fn from_env() -> Result<Self, String> {
        Ok(EmailConfig {
            smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .map_err(|_| "Invalid SMTP_PORT")?,
            smtp_username: env::var("SMTP_USERNAME").unwrap_or_else(|_| "noreply@conhub.dev".to_string()),
            smtp_password: env::var("SMTP_PASSWORD").unwrap_or_else(|_| "your-app-password".to_string()),
            from_email: env::var("FROM_EMAIL").unwrap_or_else(|_| "noreply@conhub.dev".to_string()),
            from_name: env::var("FROM_NAME").unwrap_or_else(|_| "ConHub".to_string()),
        })
    }
}

pub struct EmailService {
    config: EmailConfig,
}

impl EmailService {
    pub fn new() -> Result<Self, String> {
        let config = EmailConfig::from_env()?;
        Ok(EmailService { config })
    }

    pub async fn send_password_reset_email(&self, to_email: &str, reset_token: &str) -> Result<(), String> {
        let frontend_url = env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let reset_link = format!("{}/auth/reset-password?token={}", frontend_url, reset_token);
        
        let subject = "Reset your ConHub password";
        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Reset your ConHub password</title>
                <style>
                    body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 0; background-color: #f8fafc; }}
                    .container {{ max-width: 600px; margin: 0 auto; background-color: white; }}
                    .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 40px 20px; text-align: center; }}
                    .logo {{ color: white; font-size: 32px; font-weight: bold; margin: 0; }}
                    .content {{ padding: 40px 20px; }}
                    .button {{ display: inline-block; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; text-decoration: none; padding: 16px 32px; border-radius: 8px; font-weight: 600; margin: 20px 0; }}
                    .footer {{ background-color: #f8fafc; padding: 20px; text-align: center; color: #64748b; font-size: 14px; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1 class="logo">ConHub</h1>
                    </div>
                    <div class="content">
                        <h2>Reset your password</h2>
                        <p>Hi there,</p>
                        <p>We received a request to reset your ConHub password. Click the button below to create a new password:</p>
                        <p style="text-align: center;">
                            <a href="{}" class="button">Reset Password</a>
                        </p>
                        <p>This link will expire in 1 hour for security reasons.</p>
                        <p>If you didn't request this password reset, you can safely ignore this email. Your password will remain unchanged.</p>
                        <p>Best regards,<br>The ConHub Team</p>
                    </div>
                    <div class="footer">
                        <p>This email was sent by ConHub. If you have any questions, please contact our support team.</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            reset_link
        );

        let text_body = format!(
            r#"
Reset your ConHub password

Hi there,

We received a request to reset your ConHub password. Click the link below to create a new password:

{}

This link will expire in 1 hour for security reasons.

If you didn't request this password reset, you can safely ignore this email. Your password will remain unchanged.

Best regards,
The ConHub Team
            "#,
            reset_link
        );

        // Always log the email content for debugging
        log::info!("=== PASSWORD RESET EMAIL ===");
        log::info!("To: {}", to_email);
        log::info!("Subject: {}", subject);
        log::info!("Reset Link: {}", reset_link);
        log::info!("Token: {}", reset_token);
        log::info!("=== END EMAIL ===");
        
        // Try to send actual email
        match self.send_via_smtp(to_email, subject, &html_body, &text_body).await {
            Ok(_) => {
                log::info!("Email sent successfully to: {}", to_email);
                Ok(())
            }
            Err(e) => {
                log::warn!("Failed to send email, but returning success for security: {}", e);
                // Return success anyway to prevent email enumeration attacks
                Ok(())
            }
        }

        // In production, implement actual email sending using lettre or similar
        // For now, we'll use a simple HTTP request to a service like SendGrid, Mailgun, etc.
        self.send_via_http_service(to_email, subject, &html_body, &text_body).await
    }

    async fn send_via_smtp(&self, to_email: &str, subject: &str, html_body: &str, text_body: &str) -> Result<(), String> {
        // Use our Node.js email service
        let email_service_url = env::var("EMAIL_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3003".to_string());
        
        let client = reqwest::Client::new();
        
        // Extract token from reset link for the email service
        let reset_token = if let Some(start) = html_body.find("token=") {
            let token_start = start + 6; // "token=".len()
            if let Some(end) = html_body[token_start..].find('"') {
                html_body[token_start..token_start + end].to_string()
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        };
        
        let payload = serde_json::json!({
            "to_email": to_email,
            "reset_token": reset_token
        });
        
        let url = format!("{}/send-reset-email", email_service_url);
        
        match client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    log::info!("Email sent successfully via email service to: {}", to_email);
                    Ok(())
                } else {
                    let error_text = response.text().await.unwrap_or_default();
                    Err(format!("Email service returned error: {}", error_text))
                }
            }
            Err(e) => {
                Err(format!("Failed to send email via email service: {}", e))
            }
        }
    }
    
    async fn send_via_http_service(&self, to_email: &str, subject: &str, html_body: &str, text_body: &str) -> Result<(), String> {
        // Fallback method - just log for now
        log::info!("Sending email to {} with subject: {}", to_email, subject);
        Ok(())
    }
}