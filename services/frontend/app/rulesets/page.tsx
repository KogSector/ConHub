"use client";

import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { PlusIcon } from "@heroicons/react/24/outline";
import Link from "next/link";
import { useToast } from "@/hooks/use-toast";

interface Ruleset {
  id: string;
  name: string;
  description: string | null;
  is_active: boolean;
  created_at: string;
}

export default function RulesetsPage() {
  const [rulesets, setRulesets] = useState<Ruleset[]>([]);
  const [loading, setLoading] = useState(true);
  const { toast } = useToast();

  useEffect(() => {
    const fetchRulesets = async () => {
      try {
        const response = await fetch("/api/rulesets");
        if (!response.ok) {
          throw new Error("Failed to fetch rulesets");
        }
        const data = await response.json();
        setRulesets(data);
      } catch (error) {
        console.error("Error fetching rulesets:", error);
        toast({
          title: "Error",
          description: "Failed to load rulesets. Please try again.",
          variant: "destructive",
        });
      } finally {
        setLoading(false);
      }
    };

    fetchRulesets();
  }, [toast]);

  return (
    <div className="container mx-auto py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold">AI Agent Rulesets</h1>
        <Link href="/rulesets/new">
          <Button>
            <PlusIcon className="h-5 w-5 mr-2" />
            New Ruleset
          </Button>
        </Link>
      </div>

      {loading ? (
        <div className="flex justify-center items-center h-64">
          <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-primary"></div>
        </div>
      ) : rulesets.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <p className="text-muted-foreground mb-4">No rulesets found</p>
            <Link href="/rulesets/new">
              <Button>Create your first ruleset</Button>
            </Link>
          </CardContent>
        </Card>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {rulesets.map((ruleset) => (
            <Link key={ruleset.id} href={`/rulesets/${ruleset.id}`}>
              <Card className="h-full cursor-pointer hover:shadow-md transition-shadow">
                <CardHeader>
                  <CardTitle>{ruleset.name}</CardTitle>
                  <CardDescription>
                    {ruleset.description || "No description"}
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="flex justify-between text-sm text-muted-foreground">
                    <span>
                      Status: {ruleset.is_active ? "Active" : "Inactive"}
                    </span>
                    <span>
                      {new Date(ruleset.created_at).toLocaleDateString()}
                    </span>
                  </div>
                </CardContent>
              </Card>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}