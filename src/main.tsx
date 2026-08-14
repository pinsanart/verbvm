import ReactDOM from "react-dom/client";
import { createHashRouter } from "react-router";
import { RouterProvider } from "react-router/dom";

import "./index.css";

import Root from "./routes/Root";
import Home from "./routes/Home";
import Create from "./routes/Create";
import Review from "./routes/Review";

const router = createHashRouter([
  {
    path: "/",
    Component: Root,
    children: [
      { index: true, Component: Home },
      { path: "create", Component: Create },
      { path: "review", Component: Review },
    ]
  }
]);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <RouterProvider router={router} />
);
