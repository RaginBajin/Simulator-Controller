import { createBrowserRouter, Navigate, useRouteError } from "react-router-dom"
import { AppShell, PlaceholderPage } from "@/features/shell"
import { SessionHistoryPage } from "@/features/session-history"

function RouteErrorBoundary() {
  const error = useRouteError()
  return (
    <div className="flex h-screen items-center justify-center bg-base">
      <div className="text-center space-y-2">
        <h1 className="text-xl font-medium text-text-primary">Something went wrong</h1>
        <p className="text-sm text-text-secondary">
          {error instanceof Error ? error.message : "An unexpected error occurred."}
        </p>
      </div>
    </div>
  )
}

export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    errorElement: <RouteErrorBoundary />,
    children: [
      { index: true, element: <Navigate to="/summary" replace /> },
      {
        path: "summary",
        element: <SessionHistoryPage />,
      },
      {
        path: "coaching",
        element: <PlaceholderPage title="Coaching" />,
      },
      {
        path: "telemetry",
        element: <PlaceholderPage title="Telemetry" />,
      },
      {
        path: "session/:id",
        element: <PlaceholderPage title="Session Detail" />,
      },
    ],
  },
])
