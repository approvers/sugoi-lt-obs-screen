import "./index.css";
import "ress";

import React from "react";
import { createRoot } from "react-dom/client";

import App from "./comps/App";

const container = document.getElementById("root");
if (container == null) throw new Error("No root element");
const root = createRoot(container);

root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
