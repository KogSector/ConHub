"use client"

import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ExternalLink, FileText, Users, Calendar, Mail } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';
import { apiClient, unwrapResponse } from '@/lib/api';

interface SocialData {
  platform: string;
  data_type: string;
  title: string;
  content: string;
  url?: string;
  metadata: any;
  synced_at: string;
}

interface GroupedData {
  [platform: string]: SocialData[];
}

const PLATFORM_ICONS = {
  slack: '💬',
  notion: '📝',
  google_drive: '💾',
  gmail: '📧',
  dropbox: '📁',
  linkedin: '👔'
};

const DATA_TYPE_ICONS = {
  message: <Mail className="h-4 w-4" />,
  channel: <Users className="h-4 w-4" />,
  page: <FileText className="h-4 w-4" />,
  file: <FileText className="h-4 w-4" />,
  document: <FileText className="h-4 w-4" />,
  email: <Mail className="h-4 w-4" />,
  post: <FileText className="h-4 w-4" />,
  event: <Calendar className="h-4 w-4" />
};

export function SocialDataView() {
  const [data, setData] = useState<SocialData[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedPlatform, setSelectedPlatform] = useState<string>('all');
  const { toast } = useToast();

  useEffect(() => {
    fetchSocialData();
  }, []);

  const fetchSocialData = async () => {
    try {
      const resp = await apiClient.get('/api/social/data', {
        Authorization: `Bearer ${localStorage.getItem('token')}`,
      });

      const socialData = unwrapResponse<unknown[]>(resp) || []
      setData(socialData);
    } catch (error) {
      console.error('Error fetching social data:', error);
      toast({
        title: "Error",
        description: "Failed to fetch social data",
        variant: "destructive"
      });
    } finally {
      setLoading(false);
    }
  };

  const groupedData: GroupedData = data.reduce((acc, item) => {
    if (!acc[item.platform]) {
      acc[item.platform] = [];
    }
    acc[item.platform].push(item);
    return acc;
  }, {} as GroupedData);

  const filteredData = selectedPlatform === 'all' ? data : groupedData[selectedPlatform] || [];

  const platformTabs = Object.keys(groupedData).map(platform => ({
    value: platform,
    label: platform.charAt(0).toUpperCase() + platform.slice(1),
    count: groupedData[platform].length
  }));

  if (loading) {
    return (
      <div className="space-y-4">
        <h2 className="text-2xl font-bold">Social Data</h2>
        <div className="animate-pulse space-y-4">
          {[1, 2, 3].map(i => (
            <Card key={i}>
              <CardContent className="p-6">
                <div className="h-4 bg-gray-200 rounded w-1/4 mb-2"></div>
                <div className="h-3 bg-gray-200 rounded w-1/2 mb-2"></div>
                <div className="h-16 bg-gray-200 rounded"></div>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold mb-2">Social Data</h2>
        <p className="text-muted-foreground">
          Recent data synchronized from your connected platforms
        </p>
      </div>

      {data.length === 0 ? (
        <Card>
          <CardContent className="p-6 text-center">
            <p className="text-muted-foreground">
              No social data available. Connect platforms and sync to see data here.
            </p>
          </CardContent>
        </Card>
      ) : (
        <Tabs defaultValue="all" className="space-y-4">
          <TabsList className="grid w-full grid-cols-auto">
            <TabsTrigger 
              value="all" 
              onClick={() => setSelectedPlatform('all')}
            >
              All ({data.length})
            </TabsTrigger>
            {platformTabs.map(tab => (
              <TabsTrigger 
                key={tab.value} 
                value={tab.value}
                onClick={() => setSelectedPlatform(tab.value)}
              >
                {PLATFORM_ICONS[tab.value as keyof typeof PLATFORM_ICONS]} {tab.label} ({tab.count})
              </TabsTrigger>
            ))}
          </TabsList>

          <div className="space-y-4">
            {filteredData.map((item, index) => (
              <Card key={index} className="hover:shadow-md transition-shadow">
                <CardHeader className="pb-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-2">
                      <div className="flex items-center space-x-2">
                        {DATA_TYPE_ICONS[item.data_type as keyof typeof DATA_TYPE_ICONS] || <FileText className="h-4 w-4" />}
                        <CardTitle className="text-lg">{item.title}</CardTitle>
                      </div>
                      <Badge variant="outline">
                        {PLATFORM_ICONS[item.platform as keyof typeof PLATFORM_ICONS]} 
                        {item.platform}
                      </Badge>
                      <Badge variant="secondary">{item.data_type}</Badge>
                    </div>
                    {item.url && (
                      <Button variant="ghost" size="sm" asChild>
                        <a href={item.url} target="_blank" rel="noopener noreferrer">
                          <ExternalLink className="h-4 w-4" />
                        </a>
                      </Button>
                    )}
                  </div>
                  <CardDescription>
                    Synced {new Date(item.synced_at).toLocaleString()}
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="space-y-4">
                    <div className="prose prose-sm max-w-none">
                      <p className="text-sm text-muted-foreground line-clamp-3">
                        {item.content}
                      </p>
                    </div>
                    
                    {item.metadata && Object.keys(item.metadata).length > 0 && (
                      <div className="border-t pt-4">
                        <h4 className="text-sm font-medium mb-2">Metadata</h4>
                        <div className="grid grid-cols-2 gap-2 text-xs">
                          {Object.entries(item.metadata).map(([key, value]) => (
                            <div key={key} className="flex justify-between">
                              <span className="text-muted-foreground">{key}:</span>
                              <span className="font-mono">
                                {typeof value === 'object' ? JSON.stringify(value) : String(value)}
                              </span>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </Tabs>
      )}
    </div>
  );
}