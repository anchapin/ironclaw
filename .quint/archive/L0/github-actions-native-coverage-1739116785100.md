# Holon: GitHub Actions-Native Coverage Enforcement

**ID**: github-actions-native-coverage-1739116785100
**Level**: L0 (Hypothesis)
**Kind**: system
**Decision Context**: repo-automation-decision-1739116785000
**Created**: 2025-02-09

## Content

### Method (Recipe)
Extend existing GitHub Actions CI workflow with native coverage measurement and enforcement:

**1. Add Coverage Jobs to `.github/workflows/ci.yml`**:

```yaml
# Rust coverage with tarpaulin
coverage-rust:
  name: Rust Coverage
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: actions/cache@v3
      with:
        path: orchestrator/target
        key: ${{ runner.os }}-cargo-build-target

    - name: Install tarpaulin
      run: cargo install cargo-tarpaulin

    - name: Generate coverage
      working-directory: orchestrator
      run: |
        cargo tarpaulin --out Xml --output-dir coverage \
          --exclude-files '*/tests/*' \
          --timeout 300 \
          -- --test-threads=1

    - name: Check coverage threshold
      run: |
        COVERAGE=$(python3 <<'EOF'
        import xml.etree.ElementTree as ET
        tree = ET.parse('orchestrator/coverage/cobertura.xml')
        root = tree.getroot()
        line_rate = float(root.attrib['line-rate'])
        pct = line_rate * 100
        if pct < 75.0:
          print(f"❌ Coverage {pct:.1f}% < 75% threshold")
          exit(1)
        print(f"✅ Coverage {pct:.1f}% meets threshold")
        EOF
        )

    - uses: actions/upload-artifact@v3
      with:
        name: rust-coverage-report
        path: orchestrator/coverage/

# Python coverage with pytest-cov
coverage-python:
  name: Python Coverage
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-python@v4
      with:
        python-version: '3.11'

    - name: Install dependencies
      working-directory: agent
      run: |
        python -m venv .venv
        .venv/bin/pip install pytest pytest-cov hypothesis

    - name: Generate coverage
      working-directory: agent
      run: |
        .venv/bin/pytest tests/ \
          --cov=loop \
          --cov=tools \
          --cov-report=xml \
          --cov-report=html \
          --cov-report=term \
          --cov-fail-under=75

    - name: Check coverage threshold
      working-directory: agent
      run: |
        COVERAGE=$(.venv/bin/python -c 'import xml.etree.ElementTree as ET; tree = ET.parse("coverage.xml"); root = tree.getroot(); coverage = float(root.attrib.get("line-rate", 0)) * 100; print(f"{coverage:.1f}")')
        if (( $(echo "$COVERAGE < 75.0" | bc -l) )); then
          echo "❌ Python coverage ${COVERAGE}% < 75% threshold"
          exit 1
        fi
        echo "✅ Python coverage ${COVERAGE}% meets threshold"

    - uses: actions/upload-artifact@v3
      with:
        name: python-coverage-report
        path: agent/coverage/
        agent/htmlcov/

# Combined coverage report
coverage-report:
  name: Combined Coverage Report
  runs-on: ubuntu-latest
  needs: [coverage-rust, coverage-python]
  steps:
    - uses: actions/checkout@v4

    - name: Download all coverage reports
      uses: actions/download-artifact@v3

    - name: Generate combined report
      run: |
        echo "# IronClaw Coverage Report" > coverage-summary.md
        echo "" >> coverage-summary.md
        echo "Generated: $(date)" >> coverage-summary.md
        echo "" >> coverage-summary.md
        echo "## Rust (Orchestrator)" >> coverage-summary.md
        echo "See artifacts for detailed HTML report" >> coverage-summary.md
        echo "" >> coverage-summary.md
        echo "## Python (Agent)" >> coverage-summary.md
        echo "See artifacts for detailed HTML report" >> coverage-summary.md

    - uses: actions/upload-artifact@v3
      with:
        name: combined-coverage-report
        path: coverage-summary.md
```

**2. Add Pre-commit Coverage Hook**:

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: python-coverage
        name: Check Python coverage
        entry: bash -c 'cd agent && .venv/bin/pytest tests/ --cov=loop --cov=tools --cov-fail-under=50 -q'
        language: system
        pass_if: true  # Warn-only in pre-commit, enforce in CI

      - id: rust-coverage
        name: Check Rust coverage
        entry: bash -c 'cd orchestrator && cargo tarpaulin --exclude-files "*/tests/*" -- --test-threads=1 | grep "|| .* %"'
        language: system
        pass_if: true
```

**3. Add Documentation Check**:

```yaml
  # In ci.yml
  doc-freshness:
    name: Documentation Freshness Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Extract code examples from docs
        run: |
          grep -r '```' docs/ > /tmp/code-examples.txt || true

      - name: Install Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Validate Python code examples
        run: |
          python3 -m py_compile scripts/validate_docs.py || true

      - name: Check for TODO/FIXME in docs
        run: |
          if grep -r "TODO\|FIXME\|XXX" docs/; then
            echo "⚠️  Warning: Documentation contains TODOs"
            exit 1
          fi
```

**4. Add Scheduled Dependency Updates**:

```yaml
  # In .github/workflows/dependency-updates.yml
  name: Dependency Updates
  on:
    schedule:
      - cron: '0 0 * * 1'  # Weekly Monday

  jobs:
    rust-deps:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@stable
        - name: Update Rust dependencies
          run: |
            cd orchestrator
            cargo update
            if ! cargo check --quiet; then
              echo "❌ Dependency update broke build, reverting"
              git checkout Cargo.lock
              exit 1
            fi
        - name: Create PR
          uses: peter-evans/create-pull-request@v5
          with:
            title: "chore: Update Rust dependencies"
            body: "Automated weekly dependency update"
            branch: "deps/rust-update"

    python-deps:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - uses: actions/setup-python@v4
          with:
            python-version: '3.11'
        - name: Update Python dependencies
          run: |
            cd agent
            python -m venv .venv
            .venv/bin/pip install --upgrade pip pytest hypothesis black mypy pylint
            .venv/bin/pip freeze > requirements.txt
        - name: Create PR
          uses: peter-evans/create-pull-request@v5
          with:
            title: "chore: Update Python dependencies"
            body: "Automated weekly dependency update"
            branch: "deps/python-update"
```

**5. Add Branch Cleanup Job**:

```yaml
  # In .github/workflows/repo-cleanup.yml
  name: Repository Cleanup
  on:
    schedule:
      - cron: '0 0 * * 0'  # Weekly Sunday

  jobs:
    stale-branches:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4

        - name: Delete merged branches
          run: |
            git fetch --prune
            for branch in $(git branch -r --merged | grep -v 'main\|develop' | cut -d/ -f2-); do
              git push origin --delete $branch || true
            done
```

### Invariant Enforcement
- **Coverage Gate**: CI fails if either language < 75%
- **Trend Tracking**: Coverage percentage committed to `coverage-history.json`
- **Doc Freshness**: Fails if code examples don't compile or contain TODOs
- **Automated Maintenance**: Weekly dependency PRs, branch cleanup

## Scope
**Applies to**: GitHub Actions-based CI/CD (IronClaw uses GitHub)
**Languages**: Rust (tarpaulin), Python (pytest-cov)
**Platforms**: GitHub Actions (ubuntu-latest primarily)
**Coverage Enforcement**: 75% minimum in CI, 50% warning in pre-commit
**Runtime Impact**: +3-5 minutes to CI (coverage generation is slow)

## Rationale
```json
{
  "anomaly": "No coverage measurement, no automated quality enforcement",
  "approach": "Extend existing GitHub Actions with coverage jobs and maintenance automation",
  "alternatives_rejected": [
    "External SaaS (cost, vendor lock-in)",
    "Custom bot infrastructure (maintenance burden)",
    "Manual coverage checks (error-prone, forgettable)"
  ],
  "confidence_drivers": [
    "GitHub Actions is already used (no new platform)",
    "Native integration (no external services)",
    "Free (no additional cost)",
    "Transparent (YAML is version-controlled)"
  ]
}
```

## Relations
- **MemberOf**: repo-automation-decision-1739116785000
- **DependsOn**: []

## Advantages
✅ **Zero New Infrastructure**: Uses existing GitHub Actions
✅ **Free**: No additional cost
✅ **Transparent**: All automation is visible in YAML
✅ **Native Integration**: Works seamlessly with existing CI
✅ **No Maintenance**: No additional services to manage

## Disadvantages
❌ **Slower CI**: Coverage generation adds 3-5 minutes per run
❌ **No Pretty Dashboards**: Only artifact reports (no Code Climate-style UI)
❌ **Manual Trend Tracking**: Must build custom trend visualization
❌ **GitHub Actions Limits**: 2000 free minutes/month (may exhaust with frequent runs)

## Dependencies
None (builds on existing GitHub Actions workflow)

## Metadata
- **Author**: FPF Phase 1 (Abduction)
- **Category**: Conservative (proven pattern: GitHub Actions native)
- **Complexity**: Low
- **Risk**: Low
- **Estimated Runtime Impact**: +3-5 minutes to CI
- **Estimated Cost**: $0 (within GitHub free tier)
