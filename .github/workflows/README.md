# GitHub Actions Workflows

This directory contains the CI/CD workflows for the Marain CMS project.

## Workflows

### 1. CI (`rust.yml`)
**Trigger:** Push to main, Pull requests to main, Manual dispatch

The main CI workflow that runs on every push and pull request. It includes:
- **Formatting Check:** Ensures all Rust code follows standard formatting
- **Clippy Linting:** Static analysis to catch common mistakes and improve code quality
- **Security Audit:** Checks for known vulnerabilities in dependencies
- **Multi-platform Testing:** Runs tests on Ubuntu, macOS, and Windows
- **Frontend Checks:** TypeScript type checking and build verification
- **Tauri Build:** Builds the full application on PRs (optional)

### 2. Dependency Updates (`dependencies.yml`)
**Trigger:** Weekly (Mondays at 9am UTC), Manual dispatch

Automated dependency management:
- Updates Rust dependencies to latest versions
- Updates frontend (Bun/npm) dependencies
- Creates pull requests with the updates
- Runs security audits on all dependencies

### 3. Release (`release.yml`)
**Trigger:** Git tags matching `v*`, Manual dispatch

Automated release process:
- Creates GitHub releases
- Builds Tauri applications for all platforms:
  - Linux (x86_64) - AppImage
  - macOS (x86_64, aarch64) - DMG
  - Windows (x86_64) - MSI
- Uploads built artifacts to the release
- Supports code signing (requires secrets configuration)

### 4. Code Coverage (`coverage.yml`)
**Trigger:** Push to main, Pull requests to main

Code coverage reporting:
- Generates coverage reports using `cargo-llvm-cov`
- Uploads results to Codecov (requires `CODECOV_TOKEN` secret)
- Archives coverage reports as artifacts

## Required Secrets

Configure these in your repository settings under Settings → Secrets and variables → Actions:

- `CODECOV_TOKEN`: Token for uploading coverage reports to Codecov
- `TAURI_SIGNING_PRIVATE_KEY`: (Optional) Private key for code signing
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: (Optional) Password for the signing key

## Caching Strategy

All workflows use GitHub Actions caching to speed up builds:
- Cargo registry and index caching
- Cargo build target caching
- Node modules caching for frontend dependencies

Cache keys are unique per OS and include dependency lock file hashes to ensure cache validity.

## Platform Requirements

### Ubuntu/Linux
- GTK3 development libraries
- WebKit2GTK 4.1
- Ayatana AppIndicator3
- librsvg2

### macOS
- GTK+3
- WebKit2GTK 4.1

### Windows
- Visual Studio Build Tools (automatically available on GitHub Actions)

## Best Practices

1. **Always run `cargo fmt`** before committing Rust code
2. **Fix clippy warnings** - the CI treats warnings as errors
3. **Keep dependencies updated** - the weekly automation helps with this
4. **Write tests** - aim for good coverage of critical paths
5. **Use conventional commits** for clear history

## Troubleshooting

### Build Failures
- Check the specific job logs for detailed error messages
- Ensure all platform-specific dependencies are correctly installed
- Verify that Cargo.lock is committed and up-to-date

### Cache Issues
If you suspect cache corruption:
1. Go to Actions → Caches in your repository
2. Delete the relevant caches
3. Re-run the workflow

### Release Issues
- Ensure tags follow the `v*` pattern (e.g., `v1.0.0`)
- Check that signing secrets are properly configured if using code signing
- Verify that all platforms build successfully before creating a release

## Local Testing

To test workflows locally before pushing:

```bash
# Format check
cd src-tauri && cargo fmt --all -- --check

# Clippy
cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings

# Tests
cd src-tauri && cargo test --all

# Frontend checks
bun run check
bun run build

# Security audit
cd src-tauri && cargo audit
```

## Contributing

When adding new workflows:
1. Test thoroughly on a feature branch
2. Document the workflow purpose and triggers
3. Use consistent naming and organization
4. Leverage caching where appropriate
5. Consider the impact on CI minutes usage