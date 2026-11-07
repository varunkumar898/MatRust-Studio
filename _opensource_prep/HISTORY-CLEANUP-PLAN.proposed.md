# Safe history-cleanup plan (commands are examples; not executed)

Do not run these commands until the owner has reviewed the occurrence
inventories, selected the public repository destination, and made a protected
backup. No history rewrite, force-push, or push was performed for this audit.

## Preferred for a first public release: new repository, one clean commit

This avoids publishing the old commit graph. First sanitize the source text
files in a staging copy using
`audit-logs/privacy-working-tree.tsv`; exclude this prep folder, build output,
and any secrets. Then create a separate staging directory and new GitHub
repository:

```powershell
$source = 'C:\path\to\MATSTUDIO'
$dest = 'C:\path\to\MATSTUDIO-public'
New-Item -ItemType Directory -Path $dest
robocopy $source $dest /E /XD .git target _opensource_prep /XF .env .env.* *.pem *.key
Set-Location $dest
git init -b main
git add -A
git status --short
# Manually inspect the staged tree and re-scan it for paths, emails, and secrets.
git commit -m "Initial public release"
git remote add origin https://github.com/<OWNER>/<NEW-REPO>.git
# Only after explicit owner approval and review:
git push -u origin main
```

Use a new GitHub repository rather than pushing this new root over the
existing repository. Manually verify the `robocopy` output and remove any
unwanted ignored files, user-specific logs, or local configuration before
staging.

## Alternative: rewrite the existing repository with `git-filter-repo`

This is more disruptive. Coordinate with contributors first, pause pushes,
make two independent backups, and list all branches/tags that must be updated.
Replace the placeholders in the replacement and mailmap files with the exact
local path spellings and personal email found in the private inventory. Do
not put the actual email in a public command transcript.

```powershell
git clone --mirror https://github.com/<OWNER>/<REPO>.git C:\private\matrust-backup.git
git clone --mirror https://github.com/<OWNER>/<REPO>.git C:\private\matrust-rewrite.git
@'
C:/Users/<WINDOWS_USERNAME>==>C:/Users/<redacted>
C:\Users\<WINDOWS_USERNAME>==>C:\Users\<redacted>
<PERSONAL_EMAIL>==>redacted@example.invalid
'@ | Set-Content -Encoding ascii C:\private\replace-text.txt
@'
Public Maintainer <redacted@example.invalid> Previous Name <PERSONAL_EMAIL>
'@ | Set-Content -Encoding ascii C:\private\mailmap.txt
git -C C:\private\matrust-rewrite.git filter-repo --replace-text C:\private\replace-text.txt --mailmap C:\private\mailmap.txt
git -C C:\private\matrust-rewrite.git remote add origin https://github.com/<OWNER>/<REPO>.git
# Verify every branch and tag, file contents, and author/committer headers first.
# Only after explicit owner approval and contributor coordination:
git -C C:\private\matrust-rewrite.git push --force --mirror origin
```

`git-filter-repo` may remove the `origin` remote as a safety measure; the
example adds it back explicitly. A mirror force-push replaces remote refs and
can disrupt collaborators. GitHub forks, cached commit pages, and clones may
retain old objects; contact GitHub Support if sensitive data needs removal.
If an exposed value is a credential, revoke it before any rewrite.

After either approach, scan all refs and staged/released files again. Compare
the public result against the privacy inventory, verify notices and licenses,
and ask collaborators to clone the cleaned repository afresh.
