use crate::models::billing::*;
use crate::models::auth::SubscriptionTier;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration, Datelike};
use anyhow::{Result, anyhow};
use stripe::{
    Client, CreateCustomer, CreatePaymentIntent, CreateSubscription, CreatePrice, CreateProduct,
    Customer, PaymentIntent, Subscription, Price, Product, Invoice, PaymentMethod,
    CreatePaymentMethod, AttachPaymentMethod, CreateSetupIntent, SetupIntent,
    ListCustomers, ListSubscriptions, ListInvoices, Currency,
};
use std::collections::HashMap;

pub struct BillingService {
    stripe_client: Client,
}

impl BillingService {
    pub fn new() -> Self {
        let stripe_secret_key = std::env::var("STRIPE_SECRET_KEY")
            .unwrap_or_else(|_| "sk_test_...".to_string());
        
        let stripe_client = Client::new(stripe_secret_key);
        
        Self {
            stripe_client,
        }
    }

    pub async fn get_subscription_plans(&self) -> Result<Vec<SubscriptionPlan>> {
        
        Ok(vec![
            SubscriptionPlan {
                id: Uuid::new_v4(),
                name: "Free Plan".to_string(),
                description: Some("Perfect for getting started".to_string()),
                tier: SubscriptionTier::Free,
                price_monthly: rust_decimal::Decimal::new(0, 2),
                price_yearly: Some(rust_decimal::Decimal::new(0, 2)),
                features: serde_json::json!({"repositories": 3, "ai_queries": 100, "storage_gb": 1}),
                limits: serde_json::json!({"max_repositories": 3, "max_ai_queries_per_month": 100, "max_storage_gb": 1}),
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            SubscriptionPlan {
                id: Uuid::new_v4(),
                name: "Personal Plan".to_string(),
                description: Some("For individual developers".to_string()),
                tier: SubscriptionTier::Personal,
                price_monthly: rust_decimal::Decimal::new(1999, 2),
                price_yearly: Some(rust_decimal::Decimal::new(19999, 2)),
                features: serde_json::json!({"repositories": 20, "ai_queries": 1000, "storage_gb": 10}),
                limits: serde_json::json!({"max_repositories": 20, "max_ai_queries_per_month": 1000, "max_storage_gb": 10}),
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ])
    }

    pub async fn get_user_subscription(&self, user_id: Uuid) -> Result<Option<SubscriptionWithPlan>> {
        
        
        Ok(None)
    }

    
    pub async fn create_customer(&self, user_id: Uuid, email: &str, name: &str) -> Result<String> {
        let mut create_customer = CreateCustomer::new();
        create_customer.email = Some(email);
        create_customer.name = Some(name);
        create_customer.metadata = Some({
            let mut metadata = HashMap::new();
            metadata.insert("user_id".to_string(), user_id.to_string());
            metadata
        });

        let customer = Customer::create(&self.stripe_client, create_customer).await
            .map_err(|e| anyhow!("Failed to create Stripe customer: {}", e))?;

        Ok(customer.id.to_string())
    }

    
    pub async fn create_payment_intent(&self, amount: i64, currency: &str, customer_id: &str) -> Result<PaymentIntent> {
        let mut create_payment_intent = CreatePaymentIntent::new(amount, Currency::from_str(currency)?);
        create_payment_intent.customer = Some(customer_id.parse()?);
        create_payment_intent.automatic_payment_methods = Some(stripe::CreatePaymentIntentAutomaticPaymentMethods {
            enabled: true,
            allow_redirects: Some(stripe::CreatePaymentIntentAutomaticPaymentMethodsAllowRedirects::Never),
        });

        let payment_intent = PaymentIntent::create(&self.stripe_client, create_payment_intent).await
            .map_err(|e| anyhow!("Failed to create payment intent: {}", e))?;

        Ok(payment_intent)
    }

    
    pub async fn create_setup_intent(&self, customer_id: &str) -> Result<SetupIntent> {
        let mut create_setup_intent = CreateSetupIntent::new();
        create_setup_intent.customer = Some(customer_id.parse()?);
        create_setup_intent.usage = Some(stripe::CreateSetupIntentUsage::OffSession);

        let setup_intent = SetupIntent::create(&self.stripe_client, create_setup_intent).await
            .map_err(|e| anyhow!("Failed to create setup intent: {}", e))?;

        Ok(setup_intent)
    }

    
    pub async fn create_subscription(&self, customer_id: &str, price_id: &str) -> Result<Subscription> {
        let mut create_subscription = CreateSubscription::new(customer_id.parse()?);
        create_subscription.items = Some(vec![stripe::CreateSubscriptionItems {
            price: Some(price_id.to_string()),
            quantity: Some(1),
            ..Default::default()
        }]);
        create_subscription.payment_behavior = Some(stripe::CreateSubscriptionPaymentBehavior::DefaultIncomplete);
        create_subscription.payment_settings = Some(stripe::CreateSubscriptionPaymentSettings {
            save_default_payment_method: Some(stripe::CreateSubscriptionPaymentSettingsSaveDefaultPaymentMethod::OnSubscription),
            payment_method_options: None,
            payment_method_types: None,
        });
        create_subscription.expand = Some(vec!["latest_invoice.payment_intent".to_string()]);

        let subscription = Subscription::create(&self.stripe_client, create_subscription).await
            .map_err(|e| anyhow!("Failed to create subscription: {}", e))?;

        Ok(subscription)
    }

    
    pub async fn cancel_subscription(&self, subscription_id: &str) -> Result<Subscription> {
        let subscription = Subscription::delete(&self.stripe_client, &subscription_id.parse()?)
            .await
            .map_err(|e| anyhow!("Failed to cancel subscription: {}", e))?;

        Ok(subscription)
    }

    
    pub async fn update_subscription(&self, subscription_id: &str, new_price_id: &str) -> Result<Subscription> {
        
        let subscription = Subscription::retrieve(&self.stripe_client, &subscription_id.parse()?, &[]).await
            .map_err(|e| anyhow!("Failed to retrieve subscription: {}", e))?;

        
        let mut update_subscription = stripe::UpdateSubscription::new();
        update_subscription.items = Some(vec![stripe::UpdateSubscriptionItems {
            id: subscription.items.data.first().map(|item| item.id.clone()),
            price: Some(new_price_id.to_string()),
            quantity: Some(1),
            ..Default::default()
        }]);

        let updated_subscription = Subscription::update(&self.stripe_client, &subscription_id.parse()?, update_subscription).await
            .map_err(|e| anyhow!("Failed to update subscription: {}", e))?;

        Ok(updated_subscription)
    }

    
    pub async fn get_payment_methods(&self, customer_id: &str) -> Result<Vec<PaymentMethod>> {
        let payment_methods = PaymentMethod::list(&self.stripe_client, &stripe::ListPaymentMethods {
            customer: Some(customer_id.parse()?),
            type_: Some(stripe::PaymentMethodTypeFilter::Card),
            ..Default::default()
        }).await
        .map_err(|e| anyhow!("Failed to get payment methods: {}", e))?;

        Ok(payment_methods.data)
    }

    
    pub async fn get_invoices(&self, customer_id: &str) -> Result<Vec<Invoice>> {
        let invoices = Invoice::list(&self.stripe_client, &ListInvoices {
            customer: Some(customer_id.parse()?),
            limit: Some(10),
            ..Default::default()
        }).await
        .map_err(|e| anyhow!("Failed to get invoices: {}", e))?;

        Ok(invoices.data)
    }

    
    pub async fn handle_webhook_event(&self, payload: &str, signature: &str) -> Result<()> {
        let webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET")
            .map_err(|_| anyhow!("STRIPE_WEBHOOK_SECRET not set"))?;

        let event = stripe::Webhook::construct_event(payload, signature, &webhook_secret)
            .map_err(|e| anyhow!("Failed to construct webhook event: {}", e))?;

        match event.type_ {
            stripe::EventType::CustomerSubscriptionCreated => {
                log::info!("Subscription created: {:?}", event.data);
                
            }
            stripe::EventType::CustomerSubscriptionUpdated => {
                log::info!("Subscription updated: {:?}", event.data);
                
            }
            stripe::EventType::CustomerSubscriptionDeleted => {
                log::info!("Subscription cancelled: {:?}", event.data);
                
            }
            stripe::EventType::InvoicePaymentSucceeded => {
                log::info!("Payment succeeded: {:?}", event.data);
                
            }
            stripe::EventType::InvoicePaymentFailed => {
                log::warn!("Payment failed: {:?}", event.data);
                
            }
            _ => {
                log::debug!("Unhandled webhook event: {:?}", event.type_);
            }
        }

        Ok(())
    }

    pub async fn create_subscription(&self, _user_id: Uuid, _request: CreateSubscriptionRequest) -> Result<UserSubscription> {
        
        Ok(UserSubscription {
            id: Uuid::new_v4(),
            user_id: _user_id,
            plan_id: _request.plan_id,
            status: SubscriptionStatus::Active,
            current_period_start: Utc::now(),
            current_period_end: Utc::now() + Duration::days(30),
            trial_start: None,
            trial_end: None,
            cancel_at_period_end: false,
            cancelled_at: None,
            stripe_subscription_id: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub async fn cancel_subscription(&self, _user_id: Uuid, _cancel_at_period_end: bool) -> Result<()> {
        
        Ok(())
    }

    pub async fn get_payment_methods(&self, _user_id: Uuid) -> Result<Vec<PaymentMethod>> {
        
        Ok(vec![])
    }

    pub async fn add_payment_method(&self, _user_id: Uuid, _request: CreatePaymentMethodRequest) -> Result<PaymentMethod> {
        
        Ok(PaymentMethod {
            id: Uuid::new_v4(),
            user_id: _user_id,
            r#type: _request.r#type,
            is_default: _request.is_default.unwrap_or(false),
            stripe_payment_method_id: _request.stripe_payment_method_id,
            last_four: None,
            brand: None,
            exp_month: None,
            exp_year: None,
            billing_details: _request.billing_details.unwrap_or_else(|| serde_json::json!({})),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub async fn get_invoices(&self, _user_id: Uuid, _limit: Option<i64>) -> Result<Vec<Invoice>> {
        
        Ok(vec![])
    }

    pub async fn get_usage_tracking(&self, _user_id: Uuid, _period_start: DateTime<Utc>) -> Result<Vec<UsageTracking>> {
        
        Ok(vec![
            UsageTracking {
                id: Uuid::new_v4(),
                user_id: _user_id,
                resource_type: "repositories".to_string(),
                usage_count: 0,
                period_start: _period_start,
                period_end: _period_start + Duration::days(30),
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            UsageTracking {
                id: Uuid::new_v4(),
                user_id: _user_id,
                resource_type: "ai_queries".to_string(),
                usage_count: 0,
                period_start: _period_start,
                period_end: _period_start + Duration::days(30),
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ])
    }

    pub async fn track_usage(&self, _user_id: Uuid, _resource_type: &str, _count: i32) -> Result<()> {
        
        Ok(())
    }

    pub async fn get_billing_dashboard(&self, user_id: Uuid) -> Result<BillingDashboard> {
        let subscription = self.get_user_subscription(user_id).await?;
        let payment_methods = self.get_payment_methods(user_id).await?;
        let recent_invoices = self.get_invoices(user_id, Some(5)).await?;
        
        let current_month_start = Utc::now().date_naive().with_day(1)
            .ok_or_else(|| anyhow!("Failed to get first day of month"))?;
        let period_start = current_month_start.and_hms_opt(0, 0, 0)
            .ok_or_else(|| anyhow!("Failed to create datetime"))?
            .and_utc();
        let usage = self.get_usage_tracking(user_id, period_start).await?;

        Ok(BillingDashboard {
            subscription,
            payment_methods,
            recent_invoices,
            usage,
            billing_address: None,
        })
    }

    pub async fn check_usage_limits(&self, _user_id: Uuid, _resource_type: &str) -> Result<bool> {
        
        Ok(true)
    }
}