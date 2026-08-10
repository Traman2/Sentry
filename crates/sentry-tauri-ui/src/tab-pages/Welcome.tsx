function Welcome() {
  return (
    <div className="flex h-full w-full flex-col overflow-auto rounded-lg border border-teal bg-canvas shadow-sm">
      <div className="p-6">
        <h1 className="text-4xl font-bold text-navy">Welcome to Sentry</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          This is dummy placeholder text for the main table tab. Real process
          data will render here once the table view is wired up.
        </p>
      </div>
    </div>
  );
}

export default Welcome;
