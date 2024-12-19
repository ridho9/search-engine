import * as React from "react";
import { Link, Outlet, createRootRoute } from "@tanstack/react-router";

export const Route = createRootRoute({
  component: RootComponent,
});

const TanStackRouterDevtools =
  process.env.NODE_ENV === "production"
    ? () => null // Render nothing in production
    : React.lazy(() =>
        // Lazy load in development
        import("@tanstack/router-devtools").then((res) => ({
          default: res.TanStackRouterDevtools,
          // For Embedded Mode
          // default: res.TanStackRouterDevtoolsPanel
        }))
      );

function RootComponent() {
  return (
    <>
      <div className="p-12">
        <Outlet />
      </div>
      <React.Suspense>
        <TanStackRouterDevtools position="bottom-right" />
      </React.Suspense>
    </>
  );
}
