import { useState } from "react";

function TestPage() {
  const [count, setCount] = useState(0);

  return (
    <div className="flex h-full w-full flex-col overflow-auto rounded-lg border border-teal bg-canvas shadow-sm">
      <div className="p-6">
        <h1 className="text-4xl font-bold text-navy">Test Page</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          Dummy second tab page. Used to verify tabs swap correctly and that
          inactive tabs keep their state instead of unmounting.
        </p>
        <button
          onClick={() => setCount((c) => c + 1)}
          className="mt-4 rounded-md border border-teal px-3 py-1.5 text-sm text-navy hover:bg-teal/25"
        >
          Clicked {count} times
        </button>
      </div>
    </div>
  );
}

export default TestPage;