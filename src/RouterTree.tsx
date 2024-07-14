import { createRootRoute, createRouter } from "@tanstack/react-router";
import App from "./App";
import { AppProvider } from "./AppProvider";
import { dbRoute } from "./pages/DbPage/DbPage";
import { splashRoute } from "./pages/SplashPage/SplashPage";

export const rootRoute = createRootRoute({
  component: () => (
    <AppProvider>
      <App />
    </AppProvider>
  ),
});

const routeTree = rootRoute.addChildren([splashRoute, dbRoute]);

export const router = createRouter({
  routeTree,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
