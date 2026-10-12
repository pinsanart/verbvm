import { useTranslation } from "react-i18next";
import { LayoutDashboard } from "lucide-react";
import { EmptyState } from "../components/EmptyState";
import { Page } from "../components/Page";

export default function Dashboard() {
  const { t } = useTranslation();
  return (
    <Page title={t("nav.dashboard")}>
      <EmptyState
        icon={LayoutDashboard}
        title={t("dashboard.emptyTitle")}
        description={t("dashboard.emptyDescription")}
      />
    </Page>
  );
}
