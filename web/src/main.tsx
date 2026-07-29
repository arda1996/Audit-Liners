import React from "react";
import ReactDOM from "react-dom/client";
import { HashRouter } from "react-router-dom";
import App from "./App";
// Atelyé tipografi üçlüsü — self-host (Tauri çevrimdışı ilkesi): Jost gövde/buton,
// DM Mono veri (tutar/kod/tarih), Playfair Display başlık.
import "@fontsource/jost/400.css";
import "@fontsource/jost/500.css";
import "@fontsource/jost/600.css";
import "@fontsource/dm-mono/400.css";
import "@fontsource/dm-mono/500.css";
import "@fontsource/playfair-display/600.css";
import "@fontsource/playfair-display/700.css";
import "./styles.css";

// HashRouter: Tauri masaüstü + tarayıcı önizlemede tutarlı çalışır (asset yolu kırılmaz).
// URL şeması: /#/panel, /#/ufrs, /#/muhasebe … (frontend-routing-plani.md — kademeli göç Adım 1).
ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <HashRouter>
      <App />
    </HashRouter>
  </React.StrictMode>
);
