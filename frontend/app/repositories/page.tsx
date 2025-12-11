import { RepositoriesPageClient } from "@/components/sources/connectors/repositories";
import { AuthGuard } from "@/components/auth/AuthGuard";

export default function RepositoriesPage() {
  return (
    <AuthGuard>
      <RepositoriesPageClient />
    </AuthGuard>
  );
}