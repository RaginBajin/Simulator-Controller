import { NavLink } from "react-router-dom"

const tabs = [
  { label: "Summary", path: "/summary" },
  { label: "Coaching", path: "/coaching" },
  { label: "Telemetry", path: "/telemetry" },
] as const

export function TabBar() {
  return (
    <nav className="flex border-b border-border-default bg-surface">
      {tabs.map((tab) => (
        <NavLink
          key={tab.path}
          to={tab.path}
          className={({ isActive }) =>
            [
              "px-4 py-3 text-sm font-medium transition-colors duration-150",
              isActive
                ? "text-text-primary border-b-2 border-accent-primary"
                : "text-text-secondary hover:text-text-primary",
            ].join(" ")
          }
        >
          {tab.label}
        </NavLink>
      ))}
    </nav>
  )
}
