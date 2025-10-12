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
      // This is a placeholder. In a real application, you would fetch the branches for the given repository.
      const sampleBranches = ['main', 'develop', 'feature/new-ui', 'fix/bug-123'];
      setBranches(sampleBranches);
      setSelectedBranch(currentBranch);
    } catch (error) {
      console.error('Error fetching branches:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleBranchChange = async () => {
    if (!repositoryId || !selectedBranch) return;

    setLoading(true);
    try {
      // This is a placeholder. In a real application, you would make an API call to update the branch.
      console.log(`Changing branch for repo ${repositoryId} to ${selectedBranch}`);
      await new Promise(resolve => setTimeout(resolve, 1000)); // Simulate API call
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
