import express from 'express';
import crypto from 'crypto';

const router = express.Router();

export default function webhookRoutes(webhookService) {
  /**
   * GitHub Copilot webhook endpoint
   * POST /api/webhooks/github-copilot
   */
  router.post('/github-copilot', express.raw({ type: 'application/json' }), async (req, res) => {
    try {
      const signature = req.headers['x-hub-signature-256'];
      const event = req.headers['x-github-event'];
      const delivery = req.headers['x-github-delivery'];

      // Parse JSON body
      const body = JSON.parse(req.body.toString());

      // Process webhook
      const result = await webhookService.processWebhook('github-copilot', req.headers, body, req.body);

      res.status(200).json({
        success: true,
        event,
        delivery,
        result
      });
    } catch (error) {
      res.status(400).json({
        error: 'Webhook processing failed',
        message: error.message
      });
    }
  });

  /**
   * Amazon Q webhook endpoint
   * POST /api/webhooks/amazon-q
   */
  router.post('/amazon-q', express.raw({ type: 'application/json' }), async (req, res) => {
    try {
      const signature = req.headers['x-amz-signature'];
      const eventType = req.headers['x-amz-event-type'];
      const requestId = req.headers['x-amz-request-id'];

      // Parse JSON body
      const body = JSON.parse(req.body.toString());

      // Process webhook
      const result = await webhookService.processWebhook('amazon-q', req.headers, body, req.body);

      res.status(200).json({
        success: true,
        eventType,
        requestId,
        result
      });
    } catch (error) {
      res.status(400).json({
        error: 'Webhook processing failed',
        message: error.message
      });
    }
  });

  /**
   * Cline webhook endpoint
   * POST /api/webhooks/cline
   */
  router.post('/cline', express.raw({ type: 'application/json' }), async (req, res) => {
    try {
      const signature = req.headers['x-cline-signature'];
      const event = req.headers['x-cline-event'];
      const timestamp = req.headers['x-cline-timestamp'];

      // Parse JSON body
      const body = JSON.parse(req.body.toString());

      // Process webhook
      const result = await webhookService.processWebhook('cline', req.headers, body, req.body);

      res.status(200).json({
        success: true,
        event,
        timestamp,
        result
      });
    } catch (error) {
      res.status(400).json({
        error: 'Webhook processing failed',
        message: error.message
      });
    }
  });

  /**
   * Generic webhook endpoint for custom AI agents
   * POST /api/webhooks/generic/:agentType
   */
  router.post('/generic/:agentType', express.raw({ type: 'application/json' }), async (req, res) => {
    try {
      const { agentType } = req.params;
      const signature = req.headers['x-webhook-signature'];
      const eventType = req.headers['x-event-type'];

      // Parse JSON body
      const body = JSON.parse(req.body.toString());

      // Process webhook
      const result = await webhookService.processWebhook(agentType, req.headers, body, req.body);

      res.status(200).json({
        success: true,
        agentType,
        eventType,
        result
      });
    } catch (error) {
      res.status(400).json({
        error: 'Webhook processing failed',
        message: error.message
      });
    }
  });

  /**
   * Test webhook endpoint for development
   * POST /api/webhooks/test
   */
  router.post('/test', async (req, res) => {
    try {
      const { agentType = 'test', eventType = 'test_event', data = {} } = req.body;

      // Create test webhook data
      const testWebhook = {
        agentType,
        eventType,
        data,
        timestamp: new Date().toISOString(),
        test: true
      };

      // Process as generic webhook
      const result = await webhookService.processWebhook('generic', {
        'x-event-type': eventType,
        'content-type': 'application/json'
      }, testWebhook, JSON.stringify(testWebhook));

      res.json({
        success: true,
        message: 'Test webhook processed',
        webhook: testWebhook,
        result
      });
    } catch (error) {
      res.status(500).json({
        error: 'Test webhook failed',
        message: error.message
      });
    }
  });

  /**
   * Get webhook statistics
   * GET /api/webhooks/stats
   */
  router.get('/stats', async (req, res) => {
    try {
      const stats = webhookService.getWebhookStats();
      res.json({
        success: true,
        stats
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to get webhook stats',
        message: error.message
      });
    }
  });

  /**
   * Get webhook health status
   * GET /api/webhooks/health
   */
  router.get('/health', async (req, res) => {
    try {
      const health = webhookService.getHealthStatus();
      res.json(health);
    } catch (error) {
      res.status(500).json({
        error: 'Failed to get health status',
        message: error.message
      });
    }
  });

  /**
   * Register custom webhook handler
   * POST /api/webhooks/handlers
   */
  router.post('/handlers', async (req, res) => {
    try {
      const { agentType, secret, signatureHeader } = req.body;

      if (!agentType) {
        return res.status(400).json({
          error: 'Agent type is required'
        });
      }

      // Note: In a real implementation, you would need to provide the actual handler function
      // This is a simplified version for demonstration
      webhookService.registerWebhookHandler(
        agentType,
        async (headers, body) => {
          return { status: 'processed', agentType, timestamp: new Date() };
        },
        secret,
        signatureHeader
      );

      res.json({
        success: true,
        message: `Webhook handler registered for ${agentType}`
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to register webhook handler',
        message: error.message
      });
    }
  });

  /**
   * Validate webhook signature (utility endpoint)
   * POST /api/webhooks/validate-signature
   */
  router.post('/validate-signature', async (req, res) => {
    try {
      const { payload, signature, secret, algorithm = 'sha256' } = req.body;

      if (!payload || !signature || !secret) {
        return res.status(400).json({
          error: 'Payload, signature, and secret are required'
        });
      }

      const isValid = webhookService.verifyWebhookSignature(payload, signature, secret, algorithm);

      res.json({
        success: true,
        valid: isValid
      });
    } catch (error) {
      res.status(500).json({
        error: 'Signature validation failed',
        message: error.message
      });
    }
  });

  return router;
}
