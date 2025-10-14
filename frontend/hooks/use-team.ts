import { useState, useEffect } from 'react';

export interface TeamMember {
  id: string;
  name: string;
  email: string;
  role: string;
  status: string;
  joined_date: string;
  last_active?: string;
}

export interface InviteTeamMemberRequest {
  email: string;
  role: string;
}

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';

export function useTeam(userId: string = 'default') {
  const [members, setMembers] = useState<TeamMember[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchMembers = async () => {
    try {
      setLoading(true);
      const response = await fetch(`${API_BASE_URL}/api/settings/${userId}/team`);
      const result = await response.json();
      
      if (result.success) {
        setMembers(result.data || []);
        setError(null);
      } else {
        setError(result.error || 'Failed to fetch team members');
      }
    } catch (err) {
      setError('Network error while fetching team members');
    } finally {
      setLoading(false);
    }
  };

  const inviteMember = async (memberData: InviteTeamMemberRequest) => {
    try {
      const response = await fetch(`${API_BASE_URL}/api/settings/${userId}/team`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(memberData),
      });
      
      const result = await response.json();
      
      if (result.success) {
        await fetchMembers(); 
        return result.data;
      } else {
        setError(result.error || 'Failed to invite team member');
        return null;
      }
    } catch (err) {
      setError('Network error while inviting team member');
      return null;
    }
  };

  const removeMember = async (memberId: string) => {
    try {
      const response = await fetch(`${API_BASE_URL}/api/settings/${userId}/team/${memberId}`, {
        method: 'DELETE',
      });
      
      const result = await response.json();
      
      if (result.success) {
        await fetchMembers(); 
        return true;
      } else {
        setError(result.error || 'Failed to remove team member');
        return false;
      }
    } catch (err) {
      setError('Network error while removing team member');
      return false;
    }
  };

  useEffect(() => {
    fetchMembers();
  }, [userId]);

  return {
    members,
    loading,
    error,
    inviteMember,
    removeMember,
    refetch: fetchMembers,
  };
}