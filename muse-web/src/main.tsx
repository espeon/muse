import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createRootRoute, createRoute, createRouter } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { AppShell } from "@/components/AppShell";
import { AuthCallback } from "@/views/AuthCallback";
import { Home } from "@/views/Home";
import { Library } from "@/views/Library";
import { Playground } from "@/views/Playground";
import { Search } from "@/views/Search";
import { AlbumDetail } from "@/views/AlbumDetail";
import { ArtistDetail } from "@/views/ArtistDetail";
import { PlaylistDetail } from "@/views/PlaylistDetail";
import { Settings } from "@/views/Settings";
import { Devices } from "@/views/Devices";
import "./index.css";

const queryClient = new QueryClient();

const rootRoute = createRootRoute({ component: AppShell });

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: Home,
});


const searchRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/search",
  component: Search,
});

const libraryRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/library",
  component: Library,
});

const playgroundRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/playground",
  component: Playground,
});

const albumRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/album/$albumId",
  component: AlbumDetail,
});

const artistRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/artist/$artistId",
  component: ArtistDetail,
});

const playlistRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/playlist/$playlistId",
  component: PlaylistDetail,
});

const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/settings",
  component: Settings,
});

const devicesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/devices",
  component: Devices,
});

const callbackRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/auth/callback",
  component: AuthCallback,
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  searchRoute,
  libraryRoute,
  playgroundRoute,
  albumRoute,
  artistRoute,
  playlistRoute,
  settingsRoute,
  devicesRoute,
  callbackRoute,
]);

const router = createRouter({ routeTree });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("#root element not found");

createRoot(rootEl).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </StrictMode>,
);

// Remove the loading splash once React has mounted.
requestAnimationFrame(() => {
  const splash = document.getElementById('splash');
  if (splash) {
    splash.classList.add('hidden');
    setTimeout(() => splash.remove(), 400);
  }
});
