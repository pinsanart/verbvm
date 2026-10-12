import ReactDOM from "react-dom/client";
import { createHashRouter } from "react-router";
import { RouterProvider } from "react-router/dom";

import "./index.css";

import Root from "./routes/Root";
import Home from "./routes/Home";
import Review from "./routes/Review";
import Create from "./routes/Create";
import CreateTypes from "./routes/CreateTypes";
import CreateFlashcards from "./routes/CreateFlashcards";
import Dashboard from "./routes/Dashboard";
import Settings from "./routes/Settings";
import LanguageTests from "./routes/LanguageTests";

const router = createHashRouter([
  {
    path: "/",
    Component: Root,
    children: [
      { index: true, Component: Home },
      { path: "review", Component: Review },
      { path: "create", Component: Create },
      { path: "create/:deckName", Component: CreateTypes },
      { path: "create/:deckName/:typeSlug", Component: CreateFlashcards },
      { path: "dashboard", Component: Dashboard },
      { path: "settings", Component: Settings },
      { path: "language_tests", Component: LanguageTests },
    ],
  },
]);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <RouterProvider router={router} />
);
