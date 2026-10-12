import { LayoutDashboard } from "lucide-react";
import { EmptyState } from "../components/EmptyState";
import { Page } from "../components/Page";

export default function Dashboard() {
  return (
    <Page title="Dashboard">
      <EmptyState
        icon={LayoutDashboard}
        title="No progress to show yet"
        description="Your review history and statistics will appear here once you start reviewing."
      />
    </Page>
  );
}
