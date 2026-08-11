---
name: release
description: Cut a new release of the Sentry desktop app — bump versions, write release notes from the actual codebase changes since the last tag, tag, and create a draft GitHub Release that the Windows build workflow attaches installers to and publishes. Use when the user runs /release or asks to cut/ship/tag a release.
---

# Release

Cuts a release: decides the version, writes real release notes (not a raw
commit dump), bumps every version file, tags, and creates a **draft** GitHub
Release. Pushing the tag triggers `.github/workflows/release-build.yml`,
which builds the Windows installer, attaches it to that draft release, and
publishes it. This skill never builds or uploads anything itself — that's
CI's job, on a real Windows runner.

## Steps

1. **Find the diff range.** Run `git tag --sort=-v:refname` and take the
   first line as the previous tag. If there's no tag yet, this is the first
   release — the "since last release" framing doesn't apply; write the notes
   as a description of what the app currently does overall instead of a
   delta.

2. **Compare the codebase at the prior release to the codebase now** —
   don't reconstruct the story commit-by-commit. `git log` walks changes in
   the order they happened, which double-counts anything that was added,
   then tweaked, then fixed across several commits, and can misread a
   revert or an intermediate approach as a real feature. Instead, diff the
   two snapshots directly:
   `git diff <previous_tag>..HEAD --stat` (or, for a first release with no
   prior tag, just read the current tree — there's nothing to diff against)
   to get the *net* set of changed files, then read those files as they
   stand now (not the diff hunks) to understand what the app can actually
   do today that it couldn't before. Group findings by area: `sentry-core`,
   `sentry-tauri-ui`, `sentry-mcp` (skip a group with nothing in it). Ignore
   pure formatting/lint-only diffs — they're not release-note-worthy.

3. **Decide the version, then confirm it with the user.** Use judgment from
   the changes (new user-facing features → minor bump, fixes/cleanup only →
   patch bump, breaking changes → major bump) but don't just pick one —
   ask the user to confirm the exact version number (AskUserQuestion, with
   your recommendation as the first/default option) before writing or
   changing anything. Versioning is a judgment call that's easy to get
   wrong quietly.

4. **Write the release notes** as Markdown: a short intro line, then
   grouped bullet sections per area from step 2. Save them to a scratch
   file (e.g. `RELEASE_NOTES.md` at the repo root — it gets removed in step
   6) rather than only holding them in your head, since later steps
   (`gh release create --notes-file`, the changelog entry) both read from
   it.

5. **Bump every version file.** From the repo root:
   - `cargo set-version --workspace <version>` — covers the 3 Rust crates
     (`sentry-core`, `sentry-mcp`, `sentry-tauri-ui/src-tauri`) in one shot.
     If this errors with "no such command", tell the user once to run
     `cargo install cargo-edit`, then retry — don't silently hand-edit the
     Cargo.tomls with sed/regex instead.
   - `npm version <version> --no-git-tag-version` inside
     `crates/sentry-tauri-ui` — covers `package.json` (and
     `crates/sentry-tauri-ui/src-tauri/tauri.conf.json`'s `version`, since
     it points at `"../package.json"` rather than holding its own copy).

6. **Update `CHANGELOG.md`** at the repo root (create it with a `#
   Changelog` heading if it doesn't exist yet). Prepend a new
   `## vX.Y.Z — <today's date>` section containing the notes from step 4,
   above whatever's already there. Then delete the scratch
   `RELEASE_NOTES.md` file used to draft them — `CHANGELOG.md` is the
   permanent copy now.

7. **Commit, tag, and confirm before pushing.** Show the user the version,
   the changed files, and the notes; get explicit go-ahead — pushing the
   tag is what fires the build, and pushing to the tracked branch is a
   shared, visible action.
   Use a heredoc for the commit message so the trailer lands as a real
   second paragraph, not a literal `\n`:
   ```
   git add -A
   git commit -m "$(cat <<'EOF'
   chore(release): vX.Y.Z

   Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>
   EOF
   )"
   git tag vX.Y.Z
   git push
   git push --tags
   ```

8. **Create the draft release**, using the same notes from step 4/6:
   ```
   gh release create vX.Y.Z --draft --title vX.Y.Z --notes-file CHANGELOG.md
   ```
   (Pass just the new section's text via a temp file if you don't want the
   whole accumulated changelog as the release body — `gh release create
   --notes-file` takes the file verbatim, so extract the new section into a
   throwaway file first if `CHANGELOG.md` already has prior releases in it.)

9. **Report back**: the version, a preview of the notes, and that pushing
   the tag just triggered `release-build.yml` — it will attach the Windows
   installer to this draft release and publish it once the build finishes.
   Check `gh run list --workflow=release-build.yml -L 1` and share the run
   URL if it's already visible.

## Notes

- If `gh` isn't authenticated, everything from step 8 onward will fail with
  an auth error — tell the user to run `gh auth login` themselves.
- This skill only ever creates a **draft** release. If `release-build.yml`
  fails partway, the draft stays as-is (visible only to repo collaborators)
  rather than shipping a broken, asset-less release — re-run the workflow
  from the Actions tab, or `gh workflow run release-build.yml -f tag=vX.Y.Z`,
  once the underlying issue is fixed.
- Don't invent a version bump strategy beyond what's described in step 3 —
  if the changes are ambiguous (e.g. a mix of a new feature and a breaking
  change), say so explicitly when asking the user to confirm, rather than
  silently picking one.
