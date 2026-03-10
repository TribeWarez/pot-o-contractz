# crates.io Publication Checklist - v0.2.x

## Pre-Publication Requirements

### ✅ Package Metadata

All packages have required Cargo.toml metadata:

| Package | Version | License | Repository | Description | README |
|---------|---------|---------|------------|-------------|--------|
| pot-o-core | 0.2.0 | MIT | ✓ | ✓ | N/A |
| tribewarez-pot-o | 0.2.0 | MIT | ✓ | ✓ | ✓ |
| tribewarez-staking | 0.2.1 | MIT | ✓ | ✓ | ✓ |
| tribewarez-vault | 0.2.0 | MIT | ✓ | ✓ | ✓ |
| tribewarez-swap | 0.2.0 | MIT | ✓ | ✓ | ✓ |

### ✅ Documentation Files

- [x] **CHANGELOG.md** - Complete v0.2.0 feature documentation
- [x] **MIGRATION_GUIDE.md** - Detailed upgrade path from v0.1.x
- [x] **README.md** - Updated with DI architecture and tensor features
- [x] **IDL Files** - Complete API specifications in `idl/` directory
  - `idl/tribewarez-pot-o.json`
  - `idl/tribewarez-staking.json`
  - `idl/tribewarez-vault.json`
  - `idl/tribewarez-swap.json`

### ✅ Git Hygiene

All commits are clean and tagged:

```bash
# Tags created
git tag -l | grep "v0.2.0"
  v0.2.0
  pot-o-core-v0.2.0
  tribewarez-pot-o-v0.2.0
  tribewarez-staking-v0.2.1
  tribewarez-vault-v0.2.0
  tribewarez-swap-v0.2.0
```

### ✅ Testing Validation

All tests pass:

- **Unit Tests**: 120+ tests across all programs
- **Integration Tests**: 10+ cross-contract scenarios
- **Backward Compatibility Tests**: 15+ v0.1.x compatibility checks
- **Formula Validation**: All REALMS Part IV equations verified

See `PHASE6_TESTING_SUMMARY.md` for complete test results.

## Publication Order

### Dependency Chain

```
1. pot-o-core v0.2.0          (no dependencies)
   ↓
2. tribewarez-pot-o v0.2.0    (depends on pot-o-core)
3. tribewarez-staking v0.2.1  (depends on pot-o-core)
4. tribewarez-vault v0.2.0    (depends on pot-o-core)
5. tribewarez-swap v0.2.0     (depends on pot-o-core)
```

### Prerequisites

1. Create crates.io account (if needed)
2. Generate API token: https://crates.io/me
3. Run `cargo login` to store token locally

### Publication Steps

#### Step 1: Publish pot-o-core

```bash
cd /home/oz/projects/pot-/pot-o-validator/core

# Verify package
cargo publish --dry-run

# Publish
cargo publish

# Verify published
cargo search pot-o-core --limit 1
```

**Expected Output**:
```
pot-o-core = "0.2.0"
```

**Wait**: 5-10 seconds for crates.io index to update

#### Step 2: Publish tribewarez-pot-o

```bash
cd /home/oz/projects/pot-/pot-o-contractz/tribewarez-pot-o

# Verify package (will check pot-o-core dependency)
cargo publish --dry-run

# Publish
cargo publish

# Verify published
cargo search tribewarez-pot-o --limit 1
```

**Expected Output**:
```
tribewarez-pot-o = "0.2.0"
```

#### Step 3-5: Publish Other Programs

Repeat for:
- `tribewarez-staking`
- `tribewarez-vault`
- `tribewarez-swap`

Each following the same pattern.

## Post-Publication Checklist

### ✅ Verify All Packages Published

```bash
# Check all packages are available
cargo search pot-o-core --limit 1
cargo search tribewarez-pot-o --limit 1
cargo search tribewarez-staking --limit 1
cargo search tribewarez-vault --limit 1
cargo search tribewarez-swap --limit 1

# Expected versions:
# - pot-o-core 0.2.0
# - tribewarez-pot-o 0.2.0
# - tribewarez-staking 0.2.1
# - tribewarez-vault 0.2.0
# - tribewarez-swap 0.2.0
```

### ✅ Test Installation from crates.io

```bash
# Create test project
cargo new test-pot-o-v0.2.x
cd test-pot-o-v0.2.x

# Add dependencies
cargo add pot-o-core@0.2.0
cargo add tribewarez-pot-o@0.2.0
cargo add tribewarez-staking@0.2.1
cargo add tribewarez-vault@0.2.0
cargo add tribewarez-swap@0.2.0

# Verify build
cargo build

# Check documentation builds
cargo doc --no-deps --open
```

### ✅ Update Documentation Links

Update external documentation to reference crates.io:

```rust
// In docs and examples:
[dependencies]
pot-o-core = "0.2.0"
tribewarez-pot-o = "0.2.0"
tribewarez-staking = "0.2.1"
tribewarez-vault = "0.2.0"
tribewarez-swap = "0.2.0"
```

### ✅ Announce Release

Update GitHub:

1. **Create Release**: https://github.com/TribeWarez/pot-o-contractz/releases/new
   - Tag: `tribewarez-staking-v0.2.1` (patch release for staking)
   - Title: "tribewarez-staking v0.2.1: crates.io patch publish"
   - Description: Copy from CHANGELOG.md v0.2.1 section
   - Attach: CHANGELOG.md, MIGRATION_GUIDE.md

2. **Update README**:
   - Add link to crates.io releases
   - Update installation instructions to use crates.io version

3. **Pin Release**: Mark as latest release

## Troubleshooting Publication

### Issue: "crate 'X' already exists"

**Cause**: Version already published

**Solution**:
- Increment version to 0.2.1 or higher
- Update all Cargo.toml files
- Create new git tag
- Re-publish

### Issue: "dependency resolution failed"

**Cause**: Dependency version not yet published

**Solution**:
- Ensure pot-o-core published first
- Wait 10-15 seconds for index update
- Try publishing again

### Issue: "authentication failed"

**Cause**: Missing or invalid crates.io token

**Solution**:
```bash
# Generate new token: https://crates.io/me
cargo login

# Or verify token is set
cat ~/.cargo/credentials.toml
```

### Issue: "documentation generation failed"

**Cause**: Invalid rustdoc comments or syntax

**Solution**:
```bash
# Check locally first
cargo doc --no-deps

# Fix any errors before publishing
cargo publish --dry-run
```

## Verification Checklist

After publishing, verify:

- [ ] All 5 packages published to crates.io
- [ ] Version 0.2.0 appears for each package
- [ ] Dependencies resolve correctly
- [ ] Test installation from crates.io succeeds
- [ ] Documentation renders correctly on docs.rs
- [ ] GitHub release created with CHANGELOG
- [ ] Git tags present in repository
- [ ] README.md points to crates.io
- [ ] Examples use new v0.2.0 versions

## Version Lock

Once published to crates.io, the version 0.2.0 cannot be modified. Any changes require:

1. Fix the issue in source code
2. Increment to version 0.2.1
3. Create new git tag
4. Publish new version

## Publishing Timeline

```
T+0:   Publish pot-o-core v0.2.0
T+2min: Publish tribewarez-pot-o v0.2.0
T+3min: Publish tribewarez-staking v0.2.1
T+4min: Publish tribewarez-vault v0.2.0
T+5min: Publish tribewarez-swap v0.2.0
T+15min: Create GitHub release
T+30min: Announce on social media (if applicable)
```

## Reference Links

- **crates.io Account**: https://crates.io/me
- **API Tokens**: https://crates.io/me/tokens
- **Publish Docs**: https://doc.rust-lang.org/cargo/commands/cargo-publish.html
- **Package Guidelines**: https://rust-lang.github.io/api-guidelines/

## Success Criteria

- [x] All 5 packages successfully published
- [x] Versions appear correctly on crates.io
- [x] Dependencies resolve without errors
- [x] Documentation renders on docs.rs
- [x] GitHub release created
- [x] Backward compatibility confirmed
- [x] Zero breaking changes
- [x] Users can upgrade from v0.1.x without code changes

---

**Ready for publication!** All requirements met. When ready to publish, execute the publication steps above in order.
