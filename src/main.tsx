import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import { BrowserRouter, Routes, Route } from "react-router";
import Dashbaord from "./components/Dashbaord";
import ConnectionForm from "./components/ConnectionForm";
createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <BrowserRouter>
      <Routes>
        <Route element={<Dashbaord />}>
          <Route index path="" element={<ConnectionForm />} />
          <Route path=":slug" element={<ConnectionForm />} />
        </Route>
      </Routes>
    </BrowserRouter>
  </StrictMode>
);