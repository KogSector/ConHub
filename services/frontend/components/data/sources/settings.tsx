import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { CompanyLogo } from "@/components/ui/CompanyLogo";
import { 
  Plus,
  Settings,
  CheckCircle,
  AlertCircle
} from "lucide-react";

export function DataSourcesSettings() {
  const dataSources = [
    {
      id: "github",
      name: "GitHub",
      description: "Connect your GitHub repositories",
      connected: true,
      accounts: 2
    },
    {
      id: "bitbucket",
      name: "Bitbucket", 
      description: "Connect your Bitbucket repositories",
      connected: false,
      accounts: 0
    },
    {
      id: "google-drive",
      name: "Google Drive",
      description: "Access files from Google Drive",
      connected: true,
      accounts: 1
    },
    {
      id: "dropbox",
      name: "Dropbox",
      description: "Access files from Dropbox",
      connected: false,
      accounts: 0
    },
    {
      id: "confluence",
      name: "Confluence",
      description: "Connect to Confluence spaces",
      connected: false,
      accounts: 0
    },
    {
      id: "notion",
      name: "Notion",
      description: "Access Notion databases and pages",
      connected: true,
      accounts: 1
    },
    {
      id: "web-crawler",
      name: "Web Crawler",
      description: "Crawl and index web pages",
      connected: false,
      accounts: 0
    }
  ];

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium text-foreground">Data Sources</h3>
        <p className="text-sm text-muted-foreground">
          Connect various data sources to enhance your AI agents with external knowledge.
        </p>
      </div>

      <div className="grid gap-4">
        {dataSources.map((source) => {
          return (
            <Card key={source.id} className="bg-card border-border">
              <CardHeader className="pb-3">
                <div className="flex flex-col space-y-4">
                  <div className="flex items-center space-x-3">
                    <div className="p-2 bg-muted rounded-lg">
                      <CompanyLogo 
                        company={source.name}
                        className="w-5 h-5"
                        size={20}
                      />
                    </div>
                    <div className="flex-1">
                      <CardTitle className="text-base font-medium text-foreground">
                        {source.name}
                      </CardTitle>
                      <CardDescription className="text-sm text-muted-foreground">
                        {source.description}
                      </CardDescription>
                    </div>
                  </div>
                  <div className="flex justify-center">
                    {source.connected ? (
                      <Badge variant="secondary" className="bg-green-100 text-green-800 border-green-200">
                        <CheckCircle className="w-3 h-3 mr-1" />
                        Connected
                      </Badge>
                    ) : (
                      <Badge variant="outline" className="text-muted-foreground">
                        <AlertCircle className="w-3 h-3 mr-1" />
                        Not Connected
                      </Badge>
                    )}
                  </div>
                </div>
              </CardHeader>
              <CardContent className="pt-0">
                <div className="flex items-center justify-between">
                  <div className="text-sm text-muted-foreground">
                    {source.connected 
                      ? `${source.accounts} account${source.accounts !== 1 ? 's' : ''} connected`
                      : 'No accounts connected'
                    }
                  </div>
                  <div className="flex items-center space-x-2">
                    {source.connected && (
                      <Button variant="outline" size="sm">
                        <Settings className="w-4 h-4 mr-1" />
                        Configure
                      </Button>
                    )}
                    <Button 
                      variant={source.connected ? "outline" : "default"} 
                      size="sm"
                    >
                      {source.connected ? (
                        <>
                          <Plus className="w-4 h-4 mr-1" />
                          Add Account
                        </>
                      ) : (
                        "Connect"
                      )}
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>

      <Card className="bg-muted/50 border-dashed border-muted-foreground/25">
        <CardContent className="flex flex-col items-center justify-center py-8">
          <Plus className="w-8 h-8 text-muted-foreground mb-2" />
          <h3 className="text-sm font-medium text-foreground mb-1">Request New Data Source</h3>
          <p className="text-xs text-muted-foreground text-center mb-4">
            Don&apos;t see the data source you need? Let us know and we&apos;ll consider adding it.
          </p>
          <Button variant="outline" size="sm">
            Request Integration
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}