import ReactDOM from "react-dom/client";
import { createHashRouter } from "react-router";
import { RouterProvider } from "react-router/dom";

import "@fontsource-variable/inter";
import "./index.css";
import "./i18n";

import Root from "./routes/Root";
import Home from "./routes/Home";
import Review from "./routes/Review";
import ReviewSession from "./routes/ReviewSession";
import Decks from "./routes/Decks";
import DeckEditor from "./routes/DeckEditor";
import Dashboard from "./routes/Dashboard";
import Settings from "./routes/Settings";

const router = createHashRouter([
  {
    path: "/",
    Component: Root,
    children: [
      { index: true, Component: Home },
      { path: "review", Component: Review },
      { path: "review/:deckName/:typeSlug", Component: ReviewSession },
      { path: "decks", Component: Decks },
      { path: "decks/:deckName", Component: DeckEditor },
      { path: "dashboard", Component: Dashboard },
      { path: "settings", Component: Settings },
    ],
  },
]);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <RouterProvider router={router} />
);
