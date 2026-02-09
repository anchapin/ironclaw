# Holon: Bot Farm Automation Strategy

**ID**: bot-farm-automation-1739116785300
**Level**: L0 (Hypothesis)
**Kind**: system
**Decision Context**: repo-automation-decision-1739116785000
**Created**: 2025-02-09

## Content

### Method (Recipe)
Deploy custom GitHub bots as GitHub Actions with separate specialized workflows:

**1. Coverage Bot (tarpaulin + pytest-cov + Codecov upload)**:

```yaml
# .github/workflows/coverage-bot.yml
name: Coverage Bot

on:
  pull_request:
    types: [opened, synchronize, reopened]
  push:
    branches: [main, develop]

permissions:
  contents: read
  pull-requests: write

jobs:
  coverage-bot:
    name: Coverage Analysis
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      # Setup Rust
      - uses: dtolnay/rust-toolchain@stable

      # Setup Python
      - uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      # Install coverage tools
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Install Python coverage
        run: pip install pytest pytest-cov coverage-badge

      # Measure Rust coverage
      - name: Generate Rust coverage
        working-directory: orchestrator
        run: |
          cargo tarpaulin \
            --out Xml \
            --out Html \
            --output-dir coverage \
            --exclude-files '*/tests/*' \
            --timeout 300 \
            -- --test-threads=1

      # Measure Python coverage
      - name: Generate Python coverage
        working-directory: agent
        run: |
          python -m venv .venv
          .venv/bin/pip install -q pytest pytest-cov coverage-badge
          .venv/bin/pytest tests/ \
            --cov=loop \
            --cov=tools \
            --cov-report=xml \
            --cov-report=html \
            --cov-report=json \
            --cov-report=term-missing

      # Generate coverage badge
      - name: Generate coverage badges
        run: |
          # Python badge
          cd agent
          coverage badge -o coverage.svg -f
          mv coverage.svg ../docs/

          # Rust badge (parse XML and generate)
          python3 <<'EOF'
          import xml.etree.ElementTree as ET
          tree = ET.parse('orchestrator/coverage/cobertura.xml')
          root = tree.getroot()
          coverage = float(root.attrib['line-rate']) * 100

          color = 'brightgreen' if coverage >= 75 else 'yellow' if coverage >= 50 else 'red'
          badge = f'https://img.shields.io/badge/coverage-{coverage:.1f}%25-{color}'

          with open('docs/rust-coverage.svg', 'w') as f:
              f.write(f'<img src="{badge}" alt="Rust coverage: {coverage:.1f}%" />')
          EOF

      # Post PR comment with coverage details
      - name: Post coverage PR comment
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');

            // Parse Python coverage
            const pythonCoverage = JSON.parse(fs.readFileSync('agent/coverage.json', 'utf8'));
            const pythonPct = pythonCoverage.totals.lines.percent;

            // Parse Rust coverage
            const rustXml = fs.readFileSync('orchestrator/coverage/cobertura.xml', 'utf8');
            const rustMatch = rustXml.match(/line-rate="([\d.]+)"/);
            const rustPct = rustMatch ? parseFloat(rustMatch[1]) * 100 : 0;

            const body = `## 📊 Coverage Report

            | Component | Coverage | Status |
            |-----------|----------|--------|
            | 🦀 Rust (Orchestrator) | ${rustPct.toFixed(1)}% | ${rustPct >= 75 ? '✅' : '⚠️'} |
            | 🐍 Python (Agent) | ${pythonPct.toFixed(1)}% | ${pythonPct >= 75 ? '✅' : '⚠️'} |
            | **Overall** | **${((rustPct + pythonPct) / 2).toFixed(1)}%** | ${(rustPct + pythonPct) / 2 >= 75 ? '✅' : '⚠️'} |

            ${rustPct < 75 || pythonPct < 75 ? '⚠️ **Below 75% threshold - please add tests**' : '✅ **Meets 75% coverage requirement**'}

            ---
            *Detailed reports available in workflow artifacts*`;

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: body
            });

      # Enforce coverage threshold
      - name: Enforce 75% threshold
        run: |
          python3 <<'EOF'
          import xml.etree.ElementTree as ET
          import json

          # Rust coverage
          tree = ET.parse('orchestrator/coverage/cobertura.xml')
          rust_pct = float(tree.getroot().attrib['line-rate']) * 100

          # Python coverage
          with open('agent/coverage.json') as f:
              python_cov = json.load(f)
          python_pct = python_cov['totals']['lines']['percent']

          overall = (rust_pct + python_pct) / 2

          print(f"Rust: {rust_pct:.1f}%, Python: {python_pct:.1f}%, Overall: {overall:.1f}%")

          if overall < 75.0:
              print(f"❌ Overall coverage {overall:.1f}% < 75% threshold")
              exit(1)

          print("✅ Coverage meets 75% threshold")
          EOF

      # Upload coverage to Codecov (optional)
      - name: Upload to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: ./orchestrator/coverage/cobertura.xml,./agent/coverage.xml
          flags: rust,python
          name: codecov-umbrella
          fail_ci_if_error: false

      # Commit coverage badges
      - name: Commit coverage badges
        run: |
          git config --local user.email "action@github.com"
          git config --local user.name "Coverage Bot"
          git add docs/*.svg
          git diff --staged --quiet || git commit -m "chore: update coverage badges [skip ci]"
          git push || true
```

**2. Dependency Bot (Dependabot + custom updates)**:

```yaml
# .github/workflows/dependency-bot.yml
name: Dependency Bot

on:
  schedule:
    - cron: '0 0 * * 1'  # Weekly Monday
  workflow_dispatch:

permissions:
  contents: write
  pull-requests: write

jobs:
  dependency-bot:
    name: Dependency Updates
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      # Rust dependency update
      - name: Update Rust dependencies
        id: rust-deps
        working-directory: orchestrator
        run: |
          cargo update
          if cargo check --quiet 2>&1 | tee /tmp/rust-check.log; then
            echo "changed=true" >> $GITHUB_OUTPUT
          else
            echo "changed=false" >> $GITHUB_OUTPUT
            git checkout Cargo.lock
          fi

      # Python dependency update
      - name: Update Python dependencies
        id: python-deps
        working-directory: agent
        run: |
          python -m venv .venv
          .venv/bin/pip install --upgrade pip pytest hypothesis black mypy pylint
          .venv/bin/pip freeze > requirements-new.txt
          if ! diff requirements.txt requirements-new.txt > /dev/null; then
            mv requirements-new.txt requirements.txt
            echo "changed=true" >> $GITHUB_OUTPUT
          else
            echo "changed=false" >> $GITHUB_OUTPUT
          fi

      # Check for security advisories
      - name: Check Rust security advisories
        run: |
          cargo install cargo-audit
          cd orchestrator
          cargo audit > /tmp/security-report.txt || true

      - name: Check Python security advisories
        run: |
          pip install safety
          cd agent
          safety check --json > /tmp/python-safety.json || true

      # Create PR if dependencies changed
      - name: Create dependency update PR
        if: steps.rust-deps.outputs.changed == 'true' || steps.python-deps.outputs.changed == 'true'
        uses: peter-evans/create-pull-request@v5
        with:
          title: "🤖 [Bot] Dependency Updates $(date +%Y-%m-%d)"
          body: |
            ## Automated Dependency Update

            ### Rust Dependencies
            $(if [ "${{ steps.rust-deps.outputs.changed }}" = "true" ]; then cat /tmp/rust-check.log; fi)

            ### Security Report
            $(cat /tmp/security-report.txt 2>/dev/null || echo "No security advisories")

            ### Python Dependencies
            $(if [ "${{ steps.python-deps.outputs.changed }}" = "true" ]; then diff -u requirements.txt requirements-new.txt || true; fi)

            **Please review and test before merging.**
          branch: "bot/dependency-update-$(date +%Y%m%d)"
          labels: "dependencies,automated"
          commit-message: "chore: update dependencies [skip ci]"
```

**3. Documentation Freshness Bot**:

```yaml
# .github/workflows/doc-bot.yml
name: Documentation Bot

on:
  pull_request:
    types: [opened, synchronize, reopened]
  push:
    branches: [main, develop]

permissions:
  contents: read
  pull-requests: write

jobs:
  doc-freshness:
    name: Documentation Freshness
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      # Check doc freshness
      - name: Validate documentation
        id: doc-check
        run: |
          python3 <<'EOF'
          import re
          import subprocess
          from pathlib import Path
          import json

          issues = []
          warnings = []

          for md_file in Path('docs').rglob('*.md'):
              content = md_file.read_text()

              # Check for TODO/FIXME/XXX
              todos = re.findall(r'(TODO|FIXME|XXX)(?::?\s+(.+))?', content, re.IGNORECASE)
              if todos:
                  for match in todos:
                      marker, text = match
                      warnings.append(f"{md_file.relative_to('.')}: {marker} - {text.strip() if text.strip() else 'no context'}")

              # Check for outdated version references
              if re.search(r'version (0\.\d+\.\d+)', content):
                  versions = re.findall(r'version (0\.\d+\.\d+)', content)
                  # Check if these match actual versions
                  for version in versions:
                      # Would need to check against Cargo.toml/pyproject.toml
                      warnings.append(f"{md_file.relative_to('.')}: References version {version}")

              # Validate code examples
              code_blocks = re.findall(r'```(\w*)\n(.*?)\n```', content, re.DOTALL)
              for lang, code in code_blocks:
                  if lang in ['python', 'py']:
                      try:
                          compile(code, '<string>', 'exec')
                      except SyntaxError as e:
                          issues.append(f"{md_file.relative_to('.')}: Invalid Python code at line {e.lineno}: {e.msg}")
                  elif lang == 'rust':
                      # Basic Rust syntax validation
                      if 'fn ' in code or 'struct ' in code:
                          # Would need rustc for full validation
                          pass

          # Check for broken internal links
          for md_file in Path('docs').rglob('*.md'):
              content = md_file.read_text()
              links = re.findall(r'\[([^\]]+)\]\(([^)]+)\)', content)
              for text, url in links:
                  if url.startswith('http'):
                      continue  # Skip external links
                  # Check if internal link target exists
                  target = Path('docs') / url
                  if not target.exists() and not Path(url).exists():
                      issues.append(f"{md_file.relative_to('.')}: Broken link to {url}")

          # Output results
          result = {
              "issues": issues,
              "warnings": warnings,
              "status": "failed" if issues else "warning" if warnings else "passed"
          }

          with open('/tmp/doc-check.json', 'w') as f:
              json.dump(result, f, indent=2)

          print(f"Documentation check: {result['status']}")
          print(f"Issues: {len(issues)}, Warnings: {len(warnings)}")
          EOF

      # Post PR comment
      - name: Post doc check results
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const result = JSON.parse(fs.readFileSync('/tmp/doc-check.json', 'utf8'));

            let body = '## 📚 Documentation Freshness Check\n\n';

            if (result.issues.length > 0) {
                body += '### ❌ Issues\n';
                result.issues.forEach(issue => {
                    body += `- ${issue}\n`;
                });
                body += '\n';
            }

            if (result.warnings.length > 0) {
                body += '### ⚠️ Warnings\n';
                result.warnings.forEach(warning => {
                    body += `- ${warning}\n`;
                });
                body += '\n';
            }

            if (result.issues.length === 0 && result.warnings.length === 0) {
                body += '✅ **All checks passed!** Documentation is fresh and valid.\n';
            }

            body += '\n---\n*This check runs on every PR to keep documentation current.*';

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: body
            });

      # Fail CI on documentation errors
      - name: Fail on documentation errors
        run: |
          python3 <<'EOF'
          import json
          with open('/tmp/doc-check.json') as f:
              result = json.load(f)
          if result['issues']:
              print("❌ Documentation has errors that must be fixed")
              exit(1)
          EOF
```

**4. Test Trend Tracking Bot**:

```yaml
# .github/workflows/test-trends.yml
name: Test Trends

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  track-trends:
    name: Track Quality Trends
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      # Run all tests and collect metrics
      - name: Run Rust tests
        working-directory: orchestrator
        run: |
          cargo test --no-fail-fast -- -Z unstable-options --format json > /tmp/rust-test-results.json 2>&1 || true

      - name: Run Python tests
        working-directory: agent
        run: |
          python -m venv .venv
          .venv/bin/pip install pytest pytest-cov pytest-json-report
          .venv/bin/pytest tests/ --cov=loop --cov=tools --json-report --json-report-file=/tmp/python-test-results.json

      # Store metrics
      - name: Store metrics
        run: |
          python3 <<'EOF'
          import json
          from datetime import datetime

          # Parse results and store in trends.json
          trends = {
              "timestamp": datetime.utcnow().isoformat(),
              "commit": os.getenv('GITHUB_SHA'),
              "rust": {
                  "tests": "...",
                  "coverage": "..."
              },
              "python": {
                  "tests": "...",
                  "coverage": "..."
              }
          }

          # Append to trends history
          try:
              with open('.quality-trends.jsonl', 'r') as f:
                  history = [json.loads(line) for line in f]
          except FileNotFoundError:
              history = []

          history.append(trends)

          with open('.quality-trends.jsonl', 'w') as f:
              for entry in history[-100:]:  # Keep last 100
                  f.write(json.dumps(entry) + '\n')
          EOF

      # Generate trend visualization
      - name: Generate trend chart
        run: |
          python3 <<'EOF'
          import json
          from pathlib import Path

          trends = []
          if Path('.quality-trends.jsonl').exists():
              with open('.quality-trends.jsonl') as f:
                  trends = [json.loads(line) for line in f]

          # Generate simple markdown table
          with open('docs/quality-trends.md', 'w') as f:
              f.write('# Quality Trends\n\n')
              f.write('| Date | Rust Coverage | Python Coverage | Overall |\n')
              f.write('|------|---------------|-----------------|--------|\n')
              for trend in trends[-20:]:
                  date = trend['timestamp'][:10]
                  f.write(f"| {date} | {trend['rust']['coverage']}% | {trend['python']['coverage']}% | ... |\n")
          EOF

      # Commit trends
      - name: Commit trends
        run: |
          git config --local user.email "action@github.com"
          git config --local user.name "Trends Bot"
          git add .quality-trends.jsonl docs/quality-trends.md
          git diff --staged --quiet || git commit -m "chore: update quality trends [skip ci]"
          git push || true
```

### Invariant Enforcement
- **Coverage PR Comments**: Every PR gets coverage breakdown comment
- **Coverage Badges**: Auto-updated badges in docs
- **Automated Dependencies**: Weekly dependency PRs
- **Doc Freshness**: PRs fail if docs have errors, warn on TODOs
- **Trend Tracking**: Quality metrics tracked over time

## Scope
**Applies to**: GitHub Actions + optional Codecov integration
**Languages**: Rust (tarpaulin), Python (pytest-cov)
**Platforms**: GitHub Actions
**Bot Strategy**: Specialized workflows for each concern
**External Services**: Codecov (optional, for fancy dashboards)

## Rationale
```json
{
  "anomaly": "Need comprehensive automation with visibility and trend tracking",
  "approach": "Deploy specialized bots for coverage, dependencies, docs, and trends",
  "alternatives_rejected": [
    "Single monolithic workflow (hard to maintain, slow)",
    "Manual automation (high effort, error-prone)",
    "External SaaS (cost, vendor lock-in)"
  ],
  "confidence_drivers": [
    "Specialized bots are easier to debug",
    "PR comments provide immediate visibility",
    "Trend tracking enables data-driven decisions",
    "Coverage badges in docs show quality at a glance"
  ]
}
```

## Relations
- **MemberOf**: repo-automation-decision-1739116785000
- **DependsOn**: []

## Advantages
✅ **High Visibility**: PR comments, badges, trends
✅ **Specialized Workflows**: Easy to debug and modify
✅ **Comprehensive**: Covers all quality concerns
✅ **Professional**: Coverage badges, trend charts
✅ **Optional Codecov**: Can integrate for advanced dashboards

## Disadvantages
❌ **Complexity**: 4+ workflows to maintain
❌ **Slower**: Multiple workflows increase CI time
❌ **PR Noise**: Bot comments on every PR
❌ **Higher Cost**: More GitHub Actions minutes

## Dependencies
None (uses GitHub Actions)

## Metadata
- **Author**: FPF Phase 1 (Abduction)
- **Category**: Aggressive (maximum automation)
- **Complexity**: High
- **Risk**: Medium
- **Estimated Runtime Impact**: +5-8 minutes to CI
- **Estimated Cost**: $0-10/month (depends on Actions usage)
- **Maintenance Burden**: Medium (4+ workflows)
