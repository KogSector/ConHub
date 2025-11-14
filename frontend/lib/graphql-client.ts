import { GraphQLClient } from 'graphql-request';

const DATA_SERVICE_URL = process.env.NEXT_PUBLIC_DATA_SERVICE_URL || 'http://localhost:3013';

export const graphqlClient = new GraphQLClient(`${DATA_SERVICE_URL}/graphql`, {
  credentials: 'include',
  headers: {
    'Content-Type': 'application/json',
  },
});

// Set auth token if available
export function setAuthToken(token: string) {
  graphqlClient.setHeader('Authorization', `Bearer ${token}`);
}

// GraphQL queries and mutations
export const queries = {
  AVAILABLE_CONNECTORS: `
    query AvailableConnectors {
      availableConnectors {
        connectorType
        name
        description
        requiresOauth
        supportsWebhooks
        supportedFileTypes
      }
    }
  `,
  
  CONNECTED_ACCOUNTS: `
    query ConnectedAccounts {
      connectedAccounts {
        id
        userId
        connectorType
        accountName
        accountIdentifier
        status
        lastSyncAt
        createdAt
        updatedAt
        documentCount
        syncStats {
          totalDocuments
          indexedDocuments
          pendingDocuments
          failedDocuments
          lastSync
        }
      }
    }
  `,
  
  SOURCE_DOCUMENTS: `
    query SourceDocuments($sourceId: ID!, $limit: Int, $offset: Int) {
      sourceDocuments(sourceId: $sourceId, limit: $limit, offset: $offset) {
        id
        sourceId
        connectorType
        externalId
        name
        path
        contentType
        mimeType
        size
        url
        isFolder
        status
        createdAt
        updatedAt
        indexedAt
      }
    }
  `,
  
  EMBEDDING_QUEUE_STATUS: `
    query EmbeddingQueueStatus {
      embeddingQueueStatus {
        id
        documentId
        status
        retryCount
        errorMessage
        createdAt
        processedAt
      }
    }
  `,
};

export const mutations = {
  CONNECT_SOURCE: `
    mutation ConnectSource($input: ConnectSourceInput!) {
      connectSource(input: $input) {
        success
        message
        account {
          id
          connectorType
          accountName
          status
        }
        authorizationUrl
      }
    }
  `,
  
  COMPLETE_OAUTH: `
    mutation CompleteOAuthCallback($input: OAuthCallbackInput!) {
      completeOauthCallback(input: $input) {
        success
        message
        account {
          id
          connectorType
          accountName
          status
        }
      }
    }
  `,
  
  SYNC_SOURCE: `
    mutation SyncSource($input: SyncSourceInput!) {
      syncSource(input: $input) {
        success
        message
        documentsSynced
        documentsQueuedForEmbedding
      }
    }
  `,
  
  DISCONNECT_SOURCE: `
    mutation DisconnectSource($accountId: ID!) {
      disconnectSource(accountId: $accountId) {
        success
        message
      }
    }
  `,
  
  UPLOAD_FILE: `
    mutation UploadFile($input: UploadFileInput!) {
      uploadFile(input: $input) {
        id
        name
        status
        createdAt
      }
    }
  `,
};

export const subscriptions = {
  DATA_SOURCE_EVENTS: `
    subscription DataSourceEvents($accountId: ID) {
      dataSourceEvents(accountId: $accountId) {
        ... on ConnectionStatusEvent {
          accountId
          status
          message
        }
        ... on DocumentProcessedEvent {
          documentId
          status
          message
        }
        ... on SyncProgressEvent {
          accountId
          total
          processed
          percentage
        }
      }
    }
  `,
  
  EMBEDDING_PROGRESS: `
    subscription EmbeddingProgress {
      embeddingProgress {
        documentId
        status
        message
      }
    }
  `,
};
