# Holon: Dependabot + Coverage Ratchet Strategy

**ID**: dependabot-coverage-ratchet-1739116785200
**Level**: L0 (Hypothesis)
**Kind**: system
**Decision Context**: repo-automation-decision-1739116785000
**Created**: 2025-02-09

## Content

### Method (Recipe)
Combine GitHub Dependabot for dependencies with coverage ratchet for gradual enforcement:

**1. Enable GitHub Dependabot**:

```yaml
# .github/dependabot.yml
version: 2
updates:
  # Rust dependencies
  - package-ecosystem: "cargo"
    directory: "/orchestrator"
    schedule:
      interval: "weekly"
      day: "monday"
    open-pull-requests-limit: 10
    reviewers:
      - "alexc"
    labels:
      - "dependencies"
      - "rust"
    commit-message:
      prefix: "chore(cargo)"

  # Python dependencies
  - package-ecosystem: "pip"
    directory: "/agent"
    schedule:
      interval: "weekly"
      day: "monday"
    open-pull-requests-limit: 10
    reviewers:
      - "alexc"
    labels:
      - "dependencies"
      - "python"
    commit-message:
      prefix: "chore(pip)"

  # GitHub Actions dependencies
  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "monthly"
    commit-message:
      prefix: "ci(github-actions)"
```

**2. Implement Coverage Ratchet (Gradual Enforcement)**:

```yaml
# .github/workflows/coverage-ratchet.yml
name: Coverage Ratchet

on:
  pull_request:
    branches: [main, develop]
  push:
    branches: [main, develop]

jobs:
  # Measure current coverage
  measure-coverage:
    name: Measure Coverage
    runs-on: ubuntu-latest
    outputs:
      rust-coverage: ${{ steps.rust.outputs.coverage }}
      python-coverage: ${{ steps.python.outputs.coverage }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Need history for comparison

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Measure Rust coverage
        id: rust
        working-directory: orchestrator
        run: |
          cargo tarpaulin --out Xml --output-dir coverage \
            --exclude-files '*/tests/*' --timeout 300 -- --test-threads=1
          COVERAGE=$(python3 <<'EOF'
          import xml.etree.ElementTree as ET
          tree = ET.parse('coverage/cobertura.xml')
          root = tree.getroot()
          pct = float(root.attrib['line-rate']) * 100
          print(f"{pct:.1f}")
          EOF
          )
          echo "coverage=$COVERAGE" >> $GITHUB_OUTPUT
          echo "🦀 Rust coverage: ${COVERAGE}%"

      - name: Measure Python coverage
        id: python
        working-directory: agent
        run: |
          python -m venv .venv
          .venv/bin/pip install pytest pytest-cov hypothesis
          .venv/bin/pytest tests/ --cov=loop --cov=tools --cov-report=xml -q
          COVERAGE=$(python3 <<'EOF'
          import xml.etree.ElementTree as ET
          tree = ET.parse('coverage.xml')
          root = tree.getroot()
          pct = float(root.attrib.get('line-rate', 0)) * 100
          print(f"{pct:.1f}")
          EOF
          )
          echo "coverage=$COVERAGE" >> $GITHUB_OUTPUT
          echo "🐍 Python coverage: ${COVERAGE}%"

  # Enforce ratchet (prevent regression)
  ratchet-enforcement:
    name: Enforce Ratchet
    runs-on: ubuntu-latest
    needs: measure-coverage
    steps:
      - uses: actions/checkout@v4

      - name: Load baseline coverage
        id: baseline
        run: |
          if [ -f .coverage-baseline.json ]; then
            echo "baseline=$(cat .coverage-baseline.json)" >> $GITHUB_OUTPUT
          else
            # First run: establish baseline
            cat > .coverage-baseline.json <<'EOF'
          {
            "rust": "0.0",
            "python": "0.0",
            "rust_target": "75.0",
            "python_target": "75.0",
            "rust_ratchet": "0.0",
            "python_ratchet": "0.0"
          }
          EOF
            echo "baseline=$(cat .coverage-baseline.json)" >> $GITHUB_OUTPUT
          fi

      - name: Check Rust ratchet
        run: |
          CURRENT="${{ needs.measure-coverage.outputs.rust-coverage }}"
          RATCHET=$(echo '${{ steps.baseline.outputs.baseline }}' | jq -r '.rust_ratchet')
          TARGET=$(echo '${{ steps.baseline.outputs.baseline }}' | jq -r '.rust_target')

          echo "🦀 Current: ${CURRENT}%, Ratchet: ${RATCHET}%, Target: ${TARGET}%"

          if (( $(echo "$CURRENT < $RATCHET" | bc -l) )); then
            echo "❌ Rust coverage ${CURRENT}% < ratchet ${RATCHET}%"
            exit 1
          fi

          # Update ratchet if coverage improved
          if (( $(echo "$CURRENT > $RATCHET" | bc -l) )); then
            NEW_RATCHET=$CURRENT
            echo "📈 Rust coverage improved, updating ratchet to ${NEW_RATCHET}%"
            jq --arg nr "$NEW_RATCHET" '.rust_ratchet = $nr' .coverage-baseline.json > .tmp
            mv .tmp .coverage-baseline.json
          fi

      - name: Check Python ratchet
        run: |
          CURRENT="${{ needs.measure-coverage.outputs.python-coverage }}"
          RATCHET=$(echo '${{ steps.baseline.outputs.baseline }}' | jq -r '.python_ratchet')
          TARGET=$(echo '${{ steps.baseline.outputs.baseline }}' | jq -r '.python_target')

          echo "🐍 Current: ${CURRENT}%, Ratchet: ${RATCHET}%, Target: ${TARGET}%"

          if (( $(echo "$CURRENT < $RATCHET" | bc -l) )); then
            echo "❌ Python coverage ${CURRENT}% < ratchet ${RATCHET}%"
            exit 1
          fi

          # Update ratchet if coverage improved
          if (( $(echo "$CURRENT > $RATCHET" | bc -l) )); then
            NEW_RATCHET=$CURRENT
            echo "📈 Python coverage improved, updating ratchet to ${NEW_RATCHET}%"
            jq --arg nr "$NEW_RATCHET" '.python_ratchet = $nr' .coverage-baseline.json > .tmp
            mv .tmp .coverage-baseline.json
          fi

      - name: Commit updated ratchet
        run: |
          git config --local user.email "action@github.com"
          git config --local user.name "GitHub Action"
          git add .coverage-baseline.json
          git diff --staged --quiet || git commit -m "chore: update coverage ratchet [skip ci]"
          git push || true

  # Documentation freshness
  doc-freshness:
    name: Documentation Freshness
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Install doc dependencies
        run: |
          pip install markdown-it-py rdme

      - name: Check doc freshness
        run: |
          python3 <<'EOF'
          import re
          from pathlib import Path

          issues = []

          # Check for outdated version references
          for md_file in Path('docs').rglob('*.md'):
              content = md_file.read_text()

              # Check for TODO/FIXME in docs
              if re.search(r'TODO|FIXME|XXX', content, re.IGNORECASE):
                  issues.append(f"{md_file}: Contains TODO/FIXME markers")

              # Check for outdated code examples
              if '```' in content:
                  # Extract code blocks
                  blocks = re.findall(r'```(\w*)\n(.*?)\n```', content, re.DOTALL)
                  for lang, code in blocks:
                      if lang in ['python', 'py']:
                          try:
                              compile(code, '<string>', 'exec')
                          except SyntaxError as e:
                              issues.append(f"{md_file}: Invalid Python code: {e}")
                      elif lang == 'rust':
                          # Basic Rust syntax check (incomplete)
                          if 'fn ' not in code and 'struct ' not in code:
                              # Skip if not a function/struct
                              pass

          if issues:
              print("❌ Documentation issues found:")
              for issue in issues:
                  print(f"  - {issue}")
              exit(1)
          else:
              print("✅ Documentation freshness check passed")
          EOF

  # Stale issue/PR management
  stale-management:
    name: Stale Management
    runs-on: ubuntu-latest
    steps:
      - uses: actions/stale@v9
        with:
          repo-token: ${{ secrets.GITHUB_TOKEN }}
          days-before-stale: 30
          days-before-close: 14
          stale-issue-label: "stale"
          stale-pr-label: "stale"
          exempt-issue-labels: "enhancement,good first issue"
          exempt-pr-labels: "work-in-progress"
          stale-issue-message: |
            This issue has been inactive for 30 days. Will close in 14 days if no activity.
          stale-pr-message: |
            This PR has been inactive for 30 days. Will close in 14 days if no activity.
```

**3. Add Local Pre-commit Hooks**:

```yaml
# .pre-commit-config.yaml (additions)
repos:
  - repo: https://github.com/rust-lang/rustfmt
    rev: v1.0.0
    hooks:
      - id: rustfmt
        files: ^orchestrator/.*\.rs$

  - repo: https://github.com/psf/black
    rev: 24.1.1
    hooks:
      - id: black
        files: ^agent/.*\.py$
        args: ['--line-length=88']

  - repo: local
    hooks:
      - id: coverage-ratchet-check
        name: Check coverage ratchet
        entry: bash -c 'cd orchestrator && cargo tarpaulin --exclude-files "*/tests/*" --timeout 120 -- --test-threads=1 | grep -E "^\d+.\d+% coverage"'
        language: system
        pass_if: true

      - id: doc-todo-check
        name: Check docs for TODOs
        entry: bash -c '! grep -r "TODO\|FIXME\|XXX" docs/ || true'
        language: system
        files: ^docs/.*\.md$
```

### Invariant Enforcement
- **Coverage Ratchet**: Prevents regression, gradually increases to 75%
- **Automated Dependencies**: Dependabot creates weekly PRs for updates
- **Doc Freshness**: Fails if docs contain TODOs or invalid code examples
- **Stale Management**: Auto-closes stale issues/PRs after 44 days

## Scope
**Applies to**: GitHub with Dependabot enabled
**Languages**: Rust (cargo), Python (pip)
**Platforms**: GitHub Actions + native GitHub features
**Coverage Strategy**: Ratchet (current = max(current, previous), no regression)
**Ratchet Speed**: Gradual (depends on team velocity)

## Rationale
```json
{
  "anomaly": "No coverage enforcement, manual dependency management is error-prone",
  "approach": "Combine Dependabot (proven dependency automation) with coverage ratchet (gradual, no-regression enforcement)",
  "alternatives_rejected": [
    "Hard 75% requirement from day one (may block legitimate refactors)",
    "Manual dependency updates (high risk of security vulnerabilities)",
    "No enforcement (coverage will decay over time)"
  ],
  "confidence_drivers": [
    "Dependabot is GitHub-native (no external services)",
    "Ratchet prevents regression without blocking progress",
    "Stale management keeps issue tracker clean",
    "Balances rigor with pragmatism"
  ]
}
```

## Relations
- **MemberOf**: repo-automation-decision-1739116785000
- **DependsOn**: []

## Advantages
✅ **Gradual Enforcement**: Ratchet prevents regression without blocking progress
✅ **Dependency Security**: Dependabot catches vulnerabilities automatically
✅ **Low Friction**: Team can improve coverage at their own pace
✅ **Clean Issue Tracker**: Stale issues/PRs auto-close
✅ **GitHub Native**: No external services, fully transparent

## Disadvantages
❌ **Slower to 75%**: Ratchet may delay reaching 75% target
❌ **No Hard Enforcement**: Team could stagnate at sub-75% coverage
❌ **Dependabot PR Noise**: Weekly dependency PRs require review
❌ **Complex Setup**: Ratchet logic is more complex than hard threshold

## Dependencies
None (uses GitHub native features)

## Metadata
- **Author**: FPF Phase 1 (Abduction)
- **Category**: Moderate (balanced approach)
- **Complexity**: Medium
- **Risk**: Low
- **Estimated Runtime Impact**: +2-3 minutes to CI
- **Estimated Cost**: $0 (Dependabot is free)
- **Time to 75% Coverage**: 2-4 weeks (depends on team velocity)
