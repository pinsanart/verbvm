import { House } from "lucide-react";
import { Link } from "react-router";
import { buttonClass } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
import { Page } from "../components/Page";

export default function Home() {
  return (
    <Page title="Home">
      <EmptyState
        icon={House}
        title="Your study overview will live here"
        description="Create a deck and add flashcards to get started."
        action={
          <Link to="/create" className={buttonClass("primary")}>
            Create flashcards
          </Link>
        }
      />
    </Page>
  );
}
