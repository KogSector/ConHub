const express = require('express');
const nodemailer = require('nodemailer');
const cors = require('cors');
require('dotenv').config();

const app = express();
const PORT = process.env.EMAIL_SERVICE_PORT || 3003;

app.use(cors());
app.use(express.json());

// Create transporter for Gmail (you can change this to other providers)
const createTransporter = () => {
  return nodemailer.createTransporter({
    service: 'gmail',
    auth: {
      user: process.env.GMAIL_USER || 'your-email@gmail.com',
      pass: process.env.GMAIL_APP_PASSWORD || 'your-app-password'
    }
  });
};

// Test route
app.get('/health', (req, res) => {
  res.json({ status: 'healthy', service: 'conhub-email-service' });
});

// Send password reset email
app.post('/send-reset-email', async (req, res) => {
  try {
    const { to_email, reset_token } = req.body;
    
    if (!to_email || !reset_token) {
      return res.status(400).json({ 
        error: 'Missing required fields: to_email, reset_token' 
      });
    }

    const frontend_url = process.env.FRONTEND_URL || 'http://localhost:3000';
    const reset_link = `${frontend_url}/auth/reset-password?token=${reset_token}`;
    
    const transporter = createTransporter();
    
    const mailOptions = {
      from: {
        name: 'ConHub',
        address: process.env.GMAIL_USER || 'noreply@conhub.dev'
      },
      to: to_email,
      subject: 'Reset your ConHub password',
      html: `
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Reset your ConHub password</title>
            <style>
                body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 0; background-color: #f8fafc; }
                .container { max-width: 600px; margin: 0 auto; background-color: white; }
                .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 40px 20px; text-align: center; }
                .logo { color: white; font-size: 32px; font-weight: bold; margin: 0; }
                .content { padding: 40px 20px; }
                .button { display: inline-block; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; text-decoration: none; padding: 16px 32px; border-radius: 8px; font-weight: 600; margin: 20px 0; }
                .footer { background-color: #f8fafc; padding: 20px; text-align: center; color: #64748b; font-size: 14px; }
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
                        <a href="${reset_link}" class="button">Reset Password</a>
                    </p>
                    <p>Or copy and paste this link into your browser:</p>
                    <p style="word-break: break-all; color: #667eea;">${reset_link}</p>
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
      `,
      text: `
Reset your ConHub password

Hi there,

We received a request to reset your ConHub password. Click the link below to create a new password:

${reset_link}

This link will expire in 1 hour for security reasons.

If you didn't request this password reset, you can safely ignore this email. Your password will remain unchanged.

Best regards,
The ConHub Team
      `
    };

    const info = await transporter.sendMail(mailOptions);
    
    console.log('Password reset email sent successfully:', {
      to: to_email,
      messageId: info.messageId,
      reset_link: reset_link
    });
    
    res.json({ 
      success: true, 
      message: 'Password reset email sent successfully',
      messageId: info.messageId 
    });
    
  } catch (error) {
    console.error('Error sending email:', error);
    res.status(500).json({ 
      error: 'Failed to send email', 
      details: error.message 
    });
  }
});

app.listen(PORT, () => {
  console.log(`🚀 ConHub Email Service running on port ${PORT}`);
  console.log(`📧 Ready to send emails via Gmail`);
  
  if (!process.env.GMAIL_USER || !process.env.GMAIL_APP_PASSWORD) {
    console.warn('⚠️  Warning: GMAIL_USER and GMAIL_APP_PASSWORD not set in environment variables');
    console.log('   Please set these in your .env file to send actual emails');
  }
});