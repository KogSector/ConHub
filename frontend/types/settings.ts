export interface ApiToken {
  id: string;
  name: string;
  permissions: string[];
  createdAt: string;
  lastUsed?: string;
  token?: string; // Only present when first created
}

export interface Webhook {
  id: string;
  name: string;
  url: string;
  events: string[];
  createdAt: string;
  isActive: boolean;
  lastTriggered?: string;
}

export interface TeamMember {
  id: string;
  email: string;
  name: string;
  role: 'admin' | 'member' | 'viewer';
  status: 'active' | 'pending' | 'inactive';
  joinedAt: string;
  lastActive?: string;
}

export interface CreateApiTokenData {
  name: string;
  permissions: string[];
}

export interface CreateWebhookData {
  name: string;
  url: string;
  events: string[];
}

export interface InviteTeamMemberData {
  email: string;
  name: string;
  role: 'admin' | 'member' | 'viewer';
}