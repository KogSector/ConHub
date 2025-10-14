import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { 
  Users, 
  UserPlus, 
  Crown,
  Shield,
  Eye,
  Edit,
  Trash2,
  Mail,
  Calendar,
  MoreHorizontal,
  Settings
} from "lucide-react";

export function TeamSettings() {
  const teamMembers = [
    {
      id: 1,
      name: "John Doe",
      email: "john.doe@example.com",
      role: "owner",
      avatar: "/placeholder.svg",
      status: "active",
      joinedDate: "2024-01-15",
      lastActive: "2 minutes ago"
    },
    {
      id: 2,
      name: "Sarah Wilson",
      email: "sarah.wilson@example.com",
      role: "admin",
      avatar: "/placeholder.svg",
      status: "active",
      joinedDate: "2024-02-20",
      lastActive: "1 hour ago"
    },
    {
      id: 3,
      name: "Mike Johnson",
      email: "mike.johnson@example.com",
      role: "member",
      avatar: "/placeholder.svg",
      status: "active",
      joinedDate: "2024-03-10",
      lastActive: "1 day ago"
    },
    {
      id: 4,
      name: "Emma Davis",
      email: "emma.davis@example.com",
      role: "member",
      avatar: "/placeholder.svg",
      status: "pending",
      joinedDate: "2024-09-08",
      lastActive: "Never"
    }
  ];

  const pendingInvitations = [
    {
      id: 1,
      email: "alex.smith@example.com",
      role: "member",
      sentDate: "2024-09-07",
      expiresDate: "2024-09-14"
    },
    {
      id: 2,
      email: "lisa.brown@example.com",
      role: "admin",
      sentDate: "2024-09-06",
      expiresDate: "2024-09-13"
    }
  ];

  const getRoleIcon = (role: string) => {
    switch (role) {
      case "owner":
        return <Crown className="w-4 h-4 text-yellow-500" />;
      case "admin":
        return <Shield className="w-4 h-4 text-blue-500" />;
      default:
        return <Users className="w-4 h-4 text-gray-500" />;
    }
  };

  const getRoleBadgeVariant = (role: string) => {
    switch (role) {
      case "owner":
        return "default";
      case "admin":
        return "secondary";
      default:
        return "outline";
    }
  };

  return (
    <div className="space-y-6">
      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Users className="w-5 h-5" />
            Team Overview
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-4">
            <div className="text-center p-4 rounded-lg border border-border bg-muted/20">
              <div className="text-2xl font-bold text-primary">4</div>
              <div className="text-sm text-muted-foreground">Total Members</div>
            </div>
            <div className="text-center p-4 rounded-lg border border-border bg-muted/20">
              <div className="text-2xl font-bold text-accent">3</div>
              <div className="text-sm text-muted-foreground">Active Members</div>
            </div>
            <div className="text-center p-4 rounded-lg border border-border bg-muted/20">
              <div className="text-2xl font-bold text-primary-glow">2</div>
              <div className="text-sm text-muted-foreground">Pending Invites</div>
            </div>
            <div className="text-center p-4 rounded-lg border border-border bg-muted/20">
              <div className="text-2xl font-bold text-green-500">6</div>
              <div className="text-sm text-muted-foreground">Available Slots</div>
            </div>
          </div>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <UserPlus className="w-5 h-5" />
            Invite Team Members
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="inviteEmail">Email Address</Label>
              <Input 
                id="inviteEmail" 
                type="email"
                placeholder="colleague@example.com" 
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="inviteRole">Role</Label>
              <Select defaultValue="member">
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="member">Member</SelectItem>
                  <SelectItem value="admin">Admin</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          
          <div className="space-y-2">
            <Label>Role Permissions</Label>
            <div className="grid gap-2 md:grid-cols-2 text-sm">
              <div className="p-3 rounded-lg border border-border bg-muted/20">
                <div className="flex items-center gap-2 mb-1">
                  <Users className="w-4 h-4" />
                  <span className="font-medium">Member</span>
                </div>
                <ul className="text-xs text-muted-foreground space-y-1">
                  <li>• View repositories</li>
                  <li>• Use AI agents</li>
                  <li>• Create requests</li>
                </ul>
              </div>
              <div className="p-3 rounded-lg border border-border bg-muted/20">
                <div className="flex items-center gap-2 mb-1">
                  <Shield className="w-4 h-4" />
                  <span className="font-medium">Admin</span>
                </div>
                <ul className="text-xs text-muted-foreground space-y-1">
                  <li>• All member permissions</li>
                  <li>• Manage repositories</li>
                  <li>• Configure AI agents</li>
                  <li>• Invite team members</li>
                </ul>
              </div>
            </div>
          </div>

          <Button className="w-full md:w-auto">
            <Mail className="w-4 h-4 mr-2" />
            Send Invitation
          </Button>
        </CardContent>
      </Card>

      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Users className="w-5 h-5" />
            Team Members ({teamMembers.length})
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {teamMembers.map((member) => (
            <div key={member.id} className="p-4 rounded-lg border border-border bg-muted/20">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <Avatar className="w-10 h-10">
                    <AvatarImage src={member.avatar} alt={member.name} />
                    <AvatarFallback>
                      {member.name.split(' ').map(n => n[0]).join('')}
                    </AvatarFallback>
                  </Avatar>
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{member.name}</span>
                      {member.role === "owner" && <Crown className="w-4 h-4 text-yellow-500" />}
                      <Badge variant={getRoleBadgeVariant(member.role)} className="text-xs">
                        {member.role}
                      </Badge>
                      <Badge 
                        variant={member.status === "active" ? "default" : "secondary"}
                        className="text-xs"
                      >
                        {member.status}
                      </Badge>
                    </div>
                    <div className="text-sm text-muted-foreground">
                      {member.email}
                    </div>
                    <div className="text-xs text-muted-foreground flex items-center gap-4">
                      <span className="flex items-center gap-1">
                        <Calendar className="w-3 h-3" />
                        Joined {member.joinedDate}
                      </span>
                      <span>Last active: {member.lastActive}</span>
                    </div>
                  </div>
                </div>
                {member.role !== "owner" && (
                  <div className="flex items-center gap-2">
                    <Select defaultValue={member.role}>
                      <SelectTrigger className="w-24 h-8">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="member">Member</SelectItem>
                        <SelectItem value="admin">Admin</SelectItem>
                      </SelectContent>
                    </Select>
                    <Button variant="ghost" size="sm">
                      <Settings className="w-4 h-4" />
                    </Button>
                    <Button variant="ghost" size="sm" className="text-destructive">
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </div>
                )}
              </div>
            </div>
          ))}
        </CardContent>
      </Card>

      {}
      {pendingInvitations.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Mail className="w-5 h-5" />
              Pending Invitations ({pendingInvitations.length})
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {pendingInvitations.map((invitation) => (
              <div key={invitation.id} className="p-4 rounded-lg border border-border bg-muted/20">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <Mail className="w-5 h-5 text-muted-foreground" />
                    <div>
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{invitation.email}</span>
                        <Badge variant="outline" className="text-xs">
                          {invitation.role}
                        </Badge>
                        <Badge variant="secondary" className="text-xs">
                          Pending
                        </Badge>
                      </div>
                      <div className="text-sm text-muted-foreground">
                        Sent: {invitation.sentDate} • Expires: {invitation.expiresDate}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Button variant="outline" size="sm">
                      Resend
                    </Button>
                    <Button variant="ghost" size="sm" className="text-destructive">
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
              </div>
            ))}
          </CardContent>
        </Card>
      )}

      {}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Settings className="w-5 h-5" />
            Team Settings
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Default role for new members</Label>
              <p className="text-sm text-muted-foreground">
                Role assigned to new team members by default
              </p>
            </div>
            <Select defaultValue="member">
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="member">Member</SelectItem>
                <SelectItem value="admin">Admin</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Invitation expiry</Label>
              <p className="text-sm text-muted-foreground">
                How long invitations remain valid
              </p>
            </div>
            <Select defaultValue="7">
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="1">1 day</SelectItem>
                <SelectItem value="3">3 days</SelectItem>
                <SelectItem value="7">7 days</SelectItem>
                <SelectItem value="14">14 days</SelectItem>
                <SelectItem value="30">30 days</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Allow members to invite others</Label>
              <p className="text-sm text-muted-foreground">
                Let team members send invitations to new members
              </p>
            </div>
            <Select defaultValue="admin">
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="owner">Owner only</SelectItem>
                <SelectItem value="admin">Admins</SelectItem>
                <SelectItem value="all">All members</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>Repository access for new members</Label>
              <p className="text-sm text-muted-foreground">
                Default repository access level for new team members
              </p>
            </div>
            <Select defaultValue="read">
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">No access</SelectItem>
                <SelectItem value="read">Read only</SelectItem>
                <SelectItem value="write">Read & Write</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </CardContent>
      </Card>

      {}
      <div className="flex justify-end gap-4">
        <Button variant="outline">Cancel</Button>
        <Button>Save Changes</Button>
      </div>
    </div>
  );
}
