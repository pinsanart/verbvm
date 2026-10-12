import { BookOpen } from "lucide-react";
import { Link } from "react-router";
import { buttonClass } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
import { Page } from "../components/Page";

export default function Review() {
  return (
    <Page title="Review">
      <EmptyState
        icon={BookOpen}
        title="Nothing to review yet"
        description="Reviews are built from your flashcards. Add some to a deck and they will show up here."
        action={
          <Link to="/create" className={buttonClass("outline")}>
            Create flashcards
          </Link>
        }
      />
    </Page>
  );
}
