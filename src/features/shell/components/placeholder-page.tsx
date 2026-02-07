interface PlaceholderPageProps {
  title: string
}

export function PlaceholderPage({ title }: PlaceholderPageProps) {
  return (
    <div className="flex h-full items-center justify-center">
      <div className="text-center space-y-2">
        <h2 className="text-xl font-medium text-text-primary">{title}</h2>
        <p className="text-sm text-text-secondary">
          This section is under development.
        </p>
      </div>
    </div>
  )
}
