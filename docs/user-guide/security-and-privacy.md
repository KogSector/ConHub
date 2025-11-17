# Security and Privacy in ConHub

ConHub implements a **zero-trust security model** to protect your data and ensure granular access control.

## Zero-Trust Architecture

### What is Zero-Trust?

Traditional security models assume everything inside a network is trustworthy. Zero-trust assumes:

❌ **Never trust, always verify**
✅ **Verify explicitly**
✅ **Use least-privilege access**
✅ **Assume breach**

### How ConHub Implements Zero-Trust

#### 1. Explicit Authorization

You must explicitly authorize **every** data source:

```
✓ User authorized GitHub connection
✓ User selected repository: company/backend
✓ User granted read-only access
✓ ConHub validated user has read permissions
✓ Repository added to user's context
```

ConHub never assumes access. Just because you've connected GitHub doesn't mean we can access all your repositories.

#### 2. Read-Only Access

All OAuth connections request **read-only** permissions:

- **GitHub:** `repo:read`, `read:user` (not `repo` which includes write)
- **GitLab:** `read_api`, `read_repository` (not `api` which allows writes)
- **Google Drive:** `drive.readonly` (not `drive` which allows modifications)
- **Slack:** `channels:history`, `groups:history` (read-only)

#### 3. Granular Permissions

Access is controlled at multiple levels:

```
User → Platform → Source → Resource → Permission
```

Example:
```
john@company.com → GitHub → company/backend → /src/auth.rs → Read
```

John can only:
- Read files from `company/backend`
- Cannot access other repositories
- Cannot write or delete
- Access expires after 90 days (configurable)

#### 4. Time-Bound Access

All access grants have expiration:

```rust
AccessPolicy {
    user_id: "john@company.com",
    resource_id: "repo-123",
    permissions: [Read],
    expires_at: "2024-12-31T23:59:59Z", // Expires in 90 days
}
```

After expiration:
- Access is automatically revoked
- User must re-authorize
- Audit logs are maintained

#### 5. Conditional Access

Access can be conditional on:

```rust
AccessCondition::MfaRequired,
AccessCondition::IpWhitelist { ips: ["10.0.0.0/8"] },
AccessCondition::TimeWindow { 
    start: "09:00", 
    end: "17:00" 
},
AccessCondition::DeviceVerified,
```

Example: "Allow access to prod data only from office IPs and with MFA"

## Data Protection

### Encryption

#### At Rest
- All data encrypted with AES-256
- Separate encryption keys per customer
- Keys stored in AWS KMS / Azure Key Vault
- Regular key rotation (every 90 days)

#### In Transit
- TLS 1.3 for all connections
- Certificate pinning for API calls
- End-to-end encryption for sensitive data

### Data Storage

```
┌─────────────────────────────────────────┐
│           ConHub Architecture           │
├─────────────────────────────────────────┤
│                                         │
│  ┌──────────────┐    ┌──────────────┐  │
│  │ PostgreSQL   │    │   Qdrant     │  │
│  │ (Metadata)   │    │  (Vectors)   │  │
│  │ Encrypted    │    │  Encrypted   │  │
│  └──────────────┘    └──────────────┘  │
│                                         │
│  ┌──────────────────────────────────┐  │
│  │     Redis (Cache)                │  │
│  │     Encrypted, Short-lived       │  │
│  └──────────────────────────────────┘  │
│                                         │
└─────────────────────────────────────────┘
```

**What we store:**
- Document metadata (name, path, size, source)
- Embeddings (vector representations)
- Access policies and audit logs
- OAuth tokens (encrypted)

**What we DON'T store:**
- Your OAuth passwords
- Complete file contents (only embeddings)
- Unencrypted credentials

### Data Retention

| Data Type | Retention Period | Notes |
|-----------|-----------------|-------|
| Document embeddings | Active subscription + 30 days | Deleted on account closure |
| Audit logs | 2 years | Compliance requirement |
| OAuth tokens | Until revoked | Auto-refreshed |
| Cache data | 24 hours | Auto-expires |

### Data Location

Choose your data residency:

- **US:** AWS us-east-1 (default)
- **EU:** AWS eu-west-1 (GDPR compliant)
- **Asia:** AWS ap-southeast-1
- **On-premises:** Enterprise only

## Access Control

### User Permissions

ConHub has role-based access control (RBAC):

#### Owner
- Full access to all features
- Manage team members
- Configure security policies
- Delete workspace

#### Admin
- Manage connections
- Configure data sources
- View all documents
- Manage team members (except other admins)

#### Member
- View connected sources
- Search documents
- Use AI agent integration
- Cannot modify settings

#### Guest
- View-only access
- Cannot connect sources
- Cannot export data

### API Key Management

Secure your API keys:

```bash
# Generate new API key
curl -X POST https://api.conhub.ai/v1/keys \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name": "Production", "expires_in": 90}'

# Response
{
  "key": "chub_1234567890abcdef",
  "expires_at": "2024-12-31T23:59:59Z"
}
```

**Best practices:**
- Rotate keys every 90 days
- Use separate keys for dev/staging/prod
- Never commit keys to version control
- Use environment variables
- Revoke compromised keys immediately

### OAuth Token Security

ConHub handles OAuth tokens securely:

1. **Encrypted storage:** Tokens encrypted with customer-specific keys
2. **Automatic refresh:** Expired tokens refreshed automatically
3. **Revocation:** Instant token revocation on disconnect
4. **Scope limitation:** Minimal scopes requested

## Audit Logging

### What Gets Logged

Every action is logged:

```json
{
  "timestamp": "2024-03-15T10:30:00Z",
  "user_id": "john@company.com",
  "action": "data_source.connect",
  "resource": "github:company/backend",
  "ip_address": "203.0.113.42",
  "user_agent": "Mozilla/5.0...",
  "result": "success"
}
```

Logged events:
- User authentication
- Data source connections
- Document access
- Sync operations
- Configuration changes
- Permission modifications
- API calls

### Viewing Audit Logs

1. Navigate to **Settings** → **Audit Logs**
2. Filter by:
   - User
   - Action type
   - Date range
   - Resource
   - Result (success/failure)

### Exporting Logs

```bash
# Export audit logs
curl https://api.conhub.ai/v1/audit/export \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"start": "2024-01-01", "end": "2024-03-31"}' \
  > audit-logs.json
```

## Compliance

### Standards

ConHub complies with:

- **SOC 2 Type II:** Security, availability, processing integrity
- **GDPR:** EU data protection regulation
- **CCPA:** California privacy law
- **HIPAA:** Available for Enterprise (BAA required)
- **ISO 27001:** Information security management

### Data Processing Agreement (DPA)

ConHub acts as a **Data Processor**. You remain the **Data Controller**.

DPA includes:
- Purpose of processing
- Data types and subjects
- Security measures
- Sub-processors
- Data breach procedures

### Right to be Forgotten

GDPR compliance - delete all your data:

1. Go to **Settings** → **Account** → **Delete Account**
2. Confirm deletion
3. Within 30 days:
   - All embeddings deleted
   - All audit logs anonymized
   - OAuth tokens revoked
   - Backups expunged

## Security Best Practices

### For Users

1. **Enable MFA:** Two-factor authentication for ConHub login
2. **Use Strong Passwords:** Minimum 12 characters, mixed case, numbers, symbols
3. **Regular Audits:** Review connected sources monthly
4. **Least Privilege:** Only connect sources you need
5. **Monitor Activity:** Check audit logs for suspicious activity

### For Organizations

1. **SSO Integration:** Use SAML/OAuth for centralized auth
2. **IP Whitelisting:** Restrict access to corporate IPs
3. **Device Management:** Require verified devices
4. **Data Classification:** Mark sensitive sources
5. **Incident Response:** Have a plan for security incidents

### For Developers

1. **API Key Rotation:** Automate key rotation
2. **Secrets Management:** Use vault for credentials
3. **Least Privilege API Keys:** Create keys with minimal scopes
4. **Monitor API Usage:** Alert on unusual patterns
5. **Rate Limiting:** Implement application-level rate limits

## Incident Response

### If You Suspect a Breach

1. **Immediately:**
   - Revoke all API keys
   - Disconnect all data sources
   - Change your password
   - Enable MFA if not already enabled

2. **Contact Us:**
   - Email: security@conhub.ai
   - Include: timestamp, affected sources, description

3. **Investigation:**
   - ConHub security team investigates
   - Audit logs reviewed
   - Affected users notified (if applicable)

### ConHub's Response

If we detect a breach:

1. **Immediate:** Affected services isolated
2. **Within 1 hour:** Investigation begins
3. **Within 24 hours:** Affected customers notified
4. **Within 72 hours:** Incident report published
5. **Within 30 days:** Post-mortem and remediation

## Responsible Disclosure

Found a security vulnerability?

**Please report it responsibly:**

1. Email: security@conhub.ai
2. Do not disclose publicly until we've patched
3. Include:
   - Description of vulnerability
   - Steps to reproduce
   - Potential impact
   - Your contact information

**We will:**
- Acknowledge within 24 hours
- Provide timeline for fix
- Credit you in security advisory (if desired)
- Consider bounty for severe vulnerabilities

## FAQ

**Q: Can ConHub employees see my data?**

A: No. Data is encrypted and access is logged. Only authorized support staff with explicit customer permission can access data for troubleshooting.

**Q: What if I accidentally connected the wrong repository?**

A: Disconnect immediately. Embeddings are deleted within 24 hours. Contact support for expedited deletion.

**Q: How do I know if someone accessed my data?**

A: Check audit logs. All access is logged with IP, timestamp, and user.

**Q: Can I use ConHub with confidential code?**

A: Yes. Enterprise plans support on-premises deployment for maximum security.

**Q: Is ConHub SOC 2 compliant?**

A: Yes, SOC 2 Type II certified. Report available upon request.

**Q: What happens to my data if I cancel?**

A: Data is deleted within 30 days. You can export before canceling.

## Contact

Security questions? Contact us:

- **Email:** security@conhub.ai
- **Bug Bounty:** hackerone.com/conhub
- **Status:** status.conhub.ai

## Resources

- [Security White Paper](./security-whitepaper.pdf)
- [Compliance Certifications](./compliance/)
- [Penetration Test Results](./pentest/) (login required)
- [Security Changelog](./security-changelog.md)
