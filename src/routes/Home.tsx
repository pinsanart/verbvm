import { useTranslation } from "react-i18next";
import { House } from "lucide-react";
import { Link } from "react-router";
import { buttonClass } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
import { Page } from "../components/Page";

export default function Home() {
  const { t } = useTranslation();
  return (
    <Page title={t("nav.home")}>
      <EmptyState
        icon={House}
        title={t("home.emptyTitle")}
        description={t("home.emptyDescription")}
        action={
          <Link to="/decks" className={buttonClass("primary")}>
            {t("home.goToDecks")}
          </Link>
        }
      />
    </Page>
  );
}
