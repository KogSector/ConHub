'use client';

import { useState, useEffect } from 'react';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { apiClient, ApiResponse } from '@/lib/api';

interface ChangeBranchDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  repositoryId: string | null;
  currentBranch: string | null;
  onSuccess: () => void;
}

export function ChangeBranchDialog({
  open,
  onOpenChange,
  repositoryId,
  currentBranch,
  onSuccess,
}: ChangeBranchDialogProps) {
  const [branches, setBranches] = useState<string[]>([]);
  const [selectedBranch, setSelectedBranch] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (open && repositoryId) {
      fetchBranches(repositoryId);
    }
  }, [open, repositoryId]);

  const fetchBranches = async (repoId: string) => {
    setLoading(true);
    try {
      const resp = await apiClient.get<ApiResponse<{ branches: string[]; defaultBranch: string }>>(
        `/api/repositories/${repoId}/branches`
      );
      
      if (resp.success && resp.data) {
        setBranches(resp.data.branches || ['main', 'master', 'develop']);
        setSelectedBranch(currentBranch || resp.data.defaultBranch || 'main');
      } else {
        // Fallback to common branch names if API fails
        setBranches(['main', 'master', 'develop']);
        setSelectedBranch(currentBranch || 'main');
      }
    } catch (error) {
      console.error('Error fetching branches:', error);
      // Fallback to common branch names if API fails
      setBranches(['main', 'master', 'develop']);
      setSelectedBranch(currentBranch || 'main');
    } finally {
      setLoading(false);
    }
  };

  const handleBranchChange = async () => {
    if (!repositoryId || !selectedBranch) return;

    setLoading(true);
    try {
      
      console.log(`Changing branch for repo ${repositoryId} to ${selectedBranch}`);
      await new Promise(resolve => setTimeout(resolve, 1000)); 
      onSuccess();
      onOpenChange(false);
    } catch (error) {
      console.error('Error changing branch:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Change Branch</DialogTitle>
          <DialogDescription>
            Select a new branch to track for this repository.
          </DialogDescription>
        </DialogHeader>
        <div className="py-4">
          <Select
            value={selectedBranch || ''}
            onValueChange={setSelectedBranch}
            disabled={loading}
          >
            <SelectTrigger>
              <SelectValue placeholder="Select a branch" />
            </SelectTrigger>
            <SelectContent>
              {branches.map((branch) => (
                <SelectItem key={branch} value={branch}>
                  {branch}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={loading}>
            Cancel
          </Button>
          <Button onClick={handleBranchChange} disabled={loading || !selectedBranch}>
            {loading ? 'Saving...' : 'Save'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
