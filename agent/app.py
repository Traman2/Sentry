"""Entry point. The implementation lives in the `sentry_agent` package.

    python app.py --list-tools
    python app.py --list-models
    python app.py --ask "what's using the most CPU?"
    python app.py --serve --model qwen
"""

import sys

from sentry_agent.cli import main

if __name__ == "__main__":
    sys.exit(main())
