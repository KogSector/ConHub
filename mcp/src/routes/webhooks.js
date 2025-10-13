import express from 'express';
import crypto from 'crypto';

const router = express.Router();

export default function webhookRoutes(webhookService) {
  
  router.post('/github-copilot', express.raw({ type: 'application/json' }), async (req, res) => {
    try {
      const signature = req.headers['x-hub-signature-256'];
      const event = req.headers['x-github-event'];
      const delivery = req.headers['x-github-delivery'];

      
      const body = JSON.parse(req.body.toString());

      
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

  
  router.post('/amazon-q', express.raw({ type: 'application/json' }), async (req, res) => {
    try {
      const signature = req.headers['x-amz-signature'];
      const eventType = req.headers['x-amz-event-type'];
      const requestId = req.headers['x-amz-request-id'];

      
      const body = JSON.parse(req.body.toString());

      
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

  
  router.post('/cline', express.raw({ type: 'application/json' }), async (req, res) => {
    try {
      const signature = req.headers['x-cline-signature'];
      const event = req.headers['x-cline-event'];
      const timestamp = req.headers['x-cline-timestamp'];

      
      const body = JSON.parse(req.body.toString());

      
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

  
  router.post('/generic/:agentType', express.raw({ type: 'application/json' }), async (req, res) => {
    try {
      const { agentType } = req.params;
      const signature = req.headers['x-webhook-signature'];
      const eventType = req.headers['x-event-type'];

      
      const body = JSON.parse(req.body.toString());

      
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

  
  router.post('/test', async (req, res) => {
    try {
      const { agentType = 'test', eventType = 'test_event', data = {} } = req.body;

      
      const testWebhook = {
        agentType,
        eventType,
        data,
        timestamp: new Date().toISOString(),
        test: true
      };

      
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

  
  router.post('/handlers', async (req, res) => {
    try {
      const { agentType, secret, signatureHeader } = req.body;

      if (!agentType) {
        return res.status(400).json({
          error: 'Agent type is required'
        });
      }

      
      
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
