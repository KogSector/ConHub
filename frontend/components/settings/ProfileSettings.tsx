import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { 
  User, 
  Camera, 
  Mail, 
  Phone,
  MapPin,
  Globe,
  Github,
  Twitter,
  Linkedin
} from "lucide-react";

export function ProfileSettings() {
  return (
    <div className="space-y-6">
      {/* Profile Information */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <User className="w-5 h-5" />
            Profile Information
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Avatar Section */}
          <div className="flex items-center gap-6">
            <div className="relative">
              <Avatar className="w-24 h-24">
                <AvatarImage src="/placeholder.svg" alt="Profile" />
                <AvatarFallback>JD</AvatarFallback>
              </Avatar>
              <Button
                size="sm"
                className="absolute -bottom-2 -right-2 rounded-full p-2 h-8 w-8"
                variant="secondary"
              >
                <Camera className="w-4 h-4" />
              </Button>
            </div>
            <div className="space-y-2">
              <h3 className="text-lg font-semibold">John Doe</h3>
              <Badge variant="secondary">Pro Plan</Badge>
              <p className="text-sm text-muted-foreground">Member since March 2024</p>
            </div>
          </div>

          <Separator />

          {/* Basic Info */}
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="firstName">First Name</Label>
              <Input id="firstName" defaultValue="John" />
            </div>
            <div className="space-y-2">
              <Label htmlFor="lastName">Last Name</Label>
              <Input id="lastName" defaultValue="Doe" />
            </div>
            <div className="space-y-2">
              <Label htmlFor="email">Email</Label>
              <div className="relative">
                <Mail className="absolute left-3 top-3 w-4 h-4 text-muted-foreground" />
                <Input id="email" className="pl-10" defaultValue="john.doe@example.com" />
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="phone">Phone</Label>
              <div className="relative">
                <Phone className="absolute left-3 top-3 w-4 h-4 text-muted-foreground" />
                <Input id="phone" className="pl-10" defaultValue="+1 (555) 123-4567" />
              </div>
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="bio">Bio</Label>
            <Textarea 
              id="bio" 
              placeholder="Tell us about yourself..."
              defaultValue="Full-stack developer passionate about building scalable applications with modern technologies."
              className="resize-none"
              rows={3}
            />
          </div>

          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="location">Location</Label>
              <div className="relative">
                <MapPin className="absolute left-3 top-3 w-4 h-4 text-muted-foreground" />
                <Input id="location" className="pl-10" defaultValue="San Francisco, CA" />
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="website">Website</Label>
              <div className="relative">
                <Globe className="absolute left-3 top-3 w-4 h-4 text-muted-foreground" />
                <Input id="website" className="pl-10" placeholder="https://yourwebsite.com" />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Social Links */}
      <Card>
        <CardHeader>
          <CardTitle>Social Links</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="github">GitHub</Label>
            <div className="relative">
              <Github className="absolute left-3 top-3 w-4 h-4 text-muted-foreground" />
              <Input id="github" className="pl-10" placeholder="https://github.com/username" />
            </div>
          </div>
          <div className="space-y-2">
            <Label htmlFor="twitter">Twitter/X</Label>
            <div className="relative">
              <Twitter className="absolute left-3 top-3 w-4 h-4 text-muted-foreground" />
              <Input id="twitter" className="pl-10" placeholder="https://twitter.com/username" />
            </div>
          </div>
          <div className="space-y-2">
            <Label htmlFor="linkedin">LinkedIn</Label>
            <div className="relative">
              <Linkedin className="absolute left-3 top-3 w-4 h-4 text-muted-foreground" />
              <Input id="linkedin" className="pl-10" placeholder="https://linkedin.com/in/username" />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Save Changes */}
      <div className="flex justify-end gap-4">
        <Button variant="outline">Cancel</Button>
        <Button>Save Changes</Button>
      </div>
    </div>
  );
}
