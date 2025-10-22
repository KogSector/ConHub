"use client";

import { Suspense } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  ProfileSettings,
  NotificationSettings,
  SecuritySettings,
  BillingSettings,
  TeamSettings,
  DeveloperSettings
} from "@/components/settings";
import { ProfileAvatar } from "@/components/ui/ProfileAvatar";
import { Footer } from "@/components/ui/footer";
import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useEffect, useState } from "react";
import { SettingsProvider } from "@/contexts/settings-context";

function SettingsContent() {
  const searchParams = useSearchParams();
  const [activeTab, setActiveTab] = useState("profile");

  useEffect(() => {
    const tab = searchParams.get("tab");
    if (tab && ["profile", "notifications", "security", "billing", "team", "developer"].includes(tab)) {
      setActiveTab(tab);
    }
  }, [searchParams]);
  return (
    <div className="min-h-screen bg-background">
      {}
      <div className="border-b border-border bg-card/50 backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-20">
            <div className="flex items-center space-x-4">
              <Link href="/dashboard">
                <Button variant="ghost" size="sm">
                  <ArrowLeft className="w-4 h-4" />
                </Button>
              </Link>
              <h1 className="text-2xl font-bold text-foreground">Settings</h1>
            </div>
            <div className="flex items-center">
              <ProfileAvatar />
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <Tabs value={activeTab} onValueChange={setActiveTab} className="flex gap-8">
          {}
          <div className="w-64 flex-shrink-0">
            <TabsList className="flex flex-col h-auto w-full bg-card border border-border p-1">
              <TabsTrigger value="profile" className="w-full justify-start px-4 py-3 text-left data-[state=active]:bg-primary data-[state=active]:text-primary-foreground">
                Profile
              </TabsTrigger>
              <TabsTrigger value="notifications" className="w-full justify-start px-4 py-3 text-left data-[state=active]:bg-primary data-[state=active]:text-primary-foreground">
                Notifications
              </TabsTrigger>
              <TabsTrigger value="security" className="w-full justify-start px-4 py-3 text-left data-[state=active]:bg-primary data-[state=active]:text-primary-foreground">
                Security
              </TabsTrigger>
              <TabsTrigger value="billing" className="w-full justify-start px-4 py-3 text-left data-[state=active]:bg-primary data-[state=active]:text-primary-foreground">
                Billing
              </TabsTrigger>
              <TabsTrigger value="team" className="w-full justify-start px-4 py-3 text-left data-[state=active]:bg-primary data-[state=active]:text-primary-foreground">
                Team
              </TabsTrigger>
              <TabsTrigger value="developer" className="w-full justify-start px-4 py-3 text-left data-[state=active]:bg-primary data-[state=active]:text-primary-foreground">
                Developer
              </TabsTrigger>
            </TabsList>
          </div>

          {}
          <div className="flex-1 min-w-0">
            <TabsContent value="profile" className="mt-0 space-y-6">
              <ProfileSettings />
            </TabsContent>

            <TabsContent value="notifications" className="mt-0 space-y-6">
              <NotificationSettings />
            </TabsContent>

            <TabsContent value="security" className="mt-0 space-y-6">
              <SecuritySettings />
            </TabsContent>

            <TabsContent value="billing" className="mt-0 space-y-6">
              <BillingSettings />
            </TabsContent>

            <TabsContent value="team" className="mt-0 space-y-6">
              <TeamSettings />
            </TabsContent>

            <TabsContent value="developer" className="mt-0 space-y-6">
              <DeveloperSettings />
            </TabsContent>
          </div>
        </Tabs>
      </div>
      <Footer />
    </div>
  );
}

export default function SettingsPage() {
  return (
    <SettingsProvider>
      <Suspense fallback={<div>Loading...</div>}>
        <SettingsContent />
      </Suspense>
    </SettingsProvider>
  );
}
