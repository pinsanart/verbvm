import { Settings2 } from "lucide-react";
import { EmptyState } from "../components/EmptyState";
import { Page } from "../components/Page";

export default function Settings() {
  return (
    <Page title="Settings">
      <EmptyState
        icon={Settings2}
        title="Nothing to configure yet"
        description="Preferences for studying and for the app will be added here."
      />
    </Page>
  );
}
