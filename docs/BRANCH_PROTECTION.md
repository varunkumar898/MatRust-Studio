# Branch Protection Rules & Policy for `main`

To maintain continuous integration health and prevent regression or unauthorized modification of `main`:

## GitHub Repository Settings

Navigate to **Settings $\rightarrow$ Branches $\rightarrow$ Add branch protection rule**:
- **Branch pattern name**: `main`

### 1. Require Pull Request Before Merging
- [x] **Require approvals**: At least 1 code review approval required.
- [x] **Dismiss stale pull request approvals when new commits are pushed**: Enabled.
- [x] **Require review from Code Owners**: Enabled.

### 2. Require Status Checks to Pass Before Merging
- [x] **Require status checks to pass before merging**: Enabled.
- [x] **Require branches to be up to date before merging**: Enabled.
- **Required status checks**:
  - `Code Formatting`
  - `Build & Test (ubuntu-latest)`
  - `Build & Test (windows-latest)`
  - `Build & Test (macos-latest)`
  - `Security & License Checks`
  - `MSRV Check (Rust 1.80.0)`

### 3. Require Conversation Resolution Before Merging
- [x] All review comments and discussions must be resolved prior to merge.

### 4. Administrative Restrictions
- [x] **Do not allow bypassing the above settings**: Enabled (enforced for administrators).
- [x] **Restrict who can push to matching branches**: Merging only allowed via approved pull requests.
- [x] **Block force pushes**: Enabled (`git push --force` completely forbidden).
- [x] **Block deletions**: Enabled (`main` branch cannot be deleted).
