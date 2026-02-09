# Holon: Code Climate QaaS (Quality-as-a-Service) Strategy

**ID**: codeclimate-qaas-1739116785400
**Level**: L0 (Hypothesis)
**Kind**: system
**Decision Context**: repo-automation-decision-1739116785000
**Created**: 2025-02-09

## Content

### Method (Recipe)
Use Code Climate as external Quality-as-a-Service provider:

**1. Enable Code Climate for Repository**:

```bash
# Add Code Climate GitHub App
# Visit: https://codeclimate.com/github/<org>/<repo>/setup

# Add configuration file
cat > .codeclimate.yml <<'EOF'
---
version: "2"
plugins:
  # Rust coverage
  tarpaulin:
    enabled: true
    channel: "stable"
    config:
      exclude_files:
        - "*/tests/*"

  # Python coverage
  coverage:
    enabled: true
    config:
      exclude_files:
        - "*/tests/*"

  # Rust complexity
  complexity:
    enabled: true
    config:
      threshold: 15  # Flag functions > 15 complexity

  # Python complexity
  pep8:
    enabled: true
    config:
      max_line_length: 88

  # Security scanning
  brakeman:
    enabled: true  # For Ruby if needed later

  security:
    enabled: true
    config:
      languages:
        - python
        - rust

exclude_patterns:
  - "tests/"
  - "target/"
  - ".venv/"
  - "node_modules/"

checks:
  # Enforce 75% coverage
  coverage:
    enabled: true
    config:
      threshold: 75

  # Complexity checks
  method-complexity:
    enabled: true
    config:
      threshold: 10

  file-lines:
    enabled: true
    config:
      limit: 500  # Warn on files > 500 lines
EOF
```

**2. Add GitHub Actions Integration**:

```yaml
# .github/workflows/codeclimate.yml
name: Code Climate Analysis

on:
  pull_request:
    branches: [main, develop]
  push:
    branches: [main, develop]

jobs:
  codeclimate:
    name: Code Climate Analysis
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      # Setup tools
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      # Install Code Climate reporter
      - name: Install Code Climate reporter
        run: |
          curl -L https://codeclimate.com/downloads/test-reporter/test-reporter-latest-linux-amd64 > ./cc-test-reporter
          chmod +x ./cc-test-reporter

      # Run coverage
      - name: Run Rust coverage
        working-directory: orchestrator
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin \
            --out Xml \
            --output-dir coverage \
            --exclude-files '*/tests/*' \
            --timeout 300 \
            -- --test-threads=1

      - name: Run Python coverage
        working-directory: agent
        run: |
          python -m venv .venv
          .venv/bin/pip install pytest pytest-cov
          .venv/bin/pytest tests/ \
            --cov=loop \
            --cov=tools \
            --cov-report=xml \
            --cov-report=lcov

      # Upload to Code Climate
      - name: Format coverage for Code Climate
        run: |
          # Convert Rust coverage to lcov
          # (Code Climate prefers lcov format)

          ./cc-test-reporter format-coverage \
            --input-type lcov \
            --output coverage/codeclimate.json \
            orchestrator/coverage/lcov.info

          ./cc-test-reporter format-coverage \
            --input-type lcov \
            --output coverage/codeclimate.python.json \
            agent/coverage.lcov

      - name: Upload coverage to Code Climate
        env:
          CC_TEST_REPORTER_ID: ${{ secrets.CC_TEST_REPORTER_ID }}
        run: |
          ./cc-test-reporter upload-coverage \
            --input coverage/codeclimate.json \
            --id ${{ secrets.CC_TEST_REPORTER_ID }}

      # Post PR comment
      - name: Post Code Climate results to PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            // Code Climate automatically posts PR comments
            // This step is a placeholder for custom formatting
            console.log('Code Climate results available at: https://codeclimate.com');

  # Dependency monitoring (Code Climate includes)
  dependency-review:
    name: Dependency Review
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Rust dependency review
      - name: Rust dependency review
        uses: actions-rust-lang/rust-dependency-review@v1

      # Python dependency review
      - name: Python dependency review
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Install pip-audit
        run: pip install pip-audit

      - name: Run pip-audit
        working-directory: agent
        run: |
          python -m venv .venv
          .venv/bin/pip install pip-audit
          .venv/bin/pip-audit
```

**3. Local Development Setup**:

```yaml
# .pre-commit-config.yaml (additions)
repos:
  - repo: https://github.com/codeclimate/codeclimate-pre-commit
    rev: v0.4.0
    hooks:
      - id: codeclimate
        args:
          - --format
          - json
        pass_if: true  # Warn-only locally

  - repo: local
    hooks:
      - id: coverage-check
        name: Check coverage locally
        entry: bash -c 'make coverage && make check-coverage'
        language: system
        pass_if: true
```

**4. Makefile Targets**:

```makefile
# Makefile additions

.PHONY: coverage
coverage:
	@echo "Generating coverage reports..."
	@echo "Rust:"
	cd orchestrator && cargo tarpaulin --out Html --output-dir coverage --exclude-files '*/tests/*' --timeout 300 -- --test-threads=1
	@echo "Python:"
	cd agent && python -m venv .venv && .venv/bin/pip install pytest pytest-cov && .venv/bin/pytest tests/ --cov=loop --cov=tools --cov-report=html
	@echo "Reports generated:"
	@echo "  - orchestrator/coverage/index.html"
	@echo "  - agent/htmlcov/index.html"

.PHONY: check-coverage
check-coverage:
	@python3 <<'EOF'
	import xml.etree.ElementTree as ET
	import json

	# Rust
	try:
	    tree = ET.parse('orchestrator/coverage/cobertura.xml')
	    rust_cov = float(tree.getroot().attrib['line-rate']) * 100
	except:
	    rust_cov = 0

	# Python
	try:
	    with open('agent/coverage.json') as f:
	        python_cov = json.load(f)['totals']['lines']['percent']
	except:
	    python_cov = 0

	overall = (rust_cov + python_cov) / 2
	print(f"Rust: {rust_cov:.1f}%, Python: {python_cov:.1f}%, Overall: {overall:.1f}%")

	if overall < 75:
	    print(f"❌ Coverage {overall:.1f}% < 75% threshold")
	    exit(1)

	print("✅ Coverage meets threshold")
	EOF

.PHONY: open-coverage
open-coverage:
	@echo "Opening coverage reports..."
	@if command -v xdg-open > /dev/null; then \
		xdg-open orchestrator/coverage/index.html 2>/dev/null & \
		xdg-open agent/htmlcov/index.html 2>/dev/null & \
	elif command -v open > /dev/null; then \
		open orchestrator/coverage/index.html 2>/dev/null & \
		open agent/htmlcov/index.html 2>/dev/null & \
	fi

.PHONY: codeclimate-local
codeclimate-local:
	@echo "Running Code Climate locally..."
	@docker run --interactive --tty --rm \
	    --volume $(PWD):/code \
	    --volume /tmp/cc:/tmp/cc \
	    --volume /var/run/docker.sock:/var/run/docker.sock \
	    codeclimate/codeclimate analyze
```

**5. Documentation Checks (Lightweight)**:

```yaml
# .github/workflows/doc-check.yml
name: Documentation Check

on:
  pull_request:
    branches: [main, develop]

jobs:
  doc-check:
    name: Documentation Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      # Simple markdown linting
      - name: Lint markdown
        run: |
          pip install mdl
          mdl docs/ || true

      # Check for broken links
      - name: Check links
        run: |
          pip install markdown-link-check
          find docs/ -name '*.md' -exec markdown-link-check {} \; || true

      # Check for outdated examples
      - name: Validate code examples
        run: |
          python3 scripts/validate_doc_examples.py || true
```

### Invariant Enforcement
- **75% Coverage**: Enforced by Code Climate
- **Complexity Limits**: Functions > 15 complexity flagged
- **Security Scanning**: Automated security checks
- **Dependency Monitoring**: Automated dependency review
- **Quality Trend**: Code Climate dashboard shows trends

## Scope
**Applies to**: Projects using external Code Climate service
**Languages**: Rust (tarpaulin), Python (coverage.py)
**Platforms**: GitHub Actions + Code Climate SaaS
**Pricing**: Code Climate free tier (open source) or paid ($15+/month)
**Coverage Enforcement**: 75% via Code Climate engine

## Rationale
```json
{
  "anomaly": "Need professional quality tracking with minimal maintenance",
  "approach": "Outsource quality automation to Code Climate SaaS",
  "alternatives_rejected": [
    "Build custom automation (high maintenance burden)",
    "Use multiple disjointed tools (fragmentation)",
    "Manual quality checks (error-prone, inconsistent)"
  ],
  "confidence_drivers": [
    "Zero maintenance (Code Climate handles everything)",
    "Professional dashboards (trends, metrics)",
    "Industry standard (many teams use it)",
    "GitHub integration (PR comments, status checks)"
  ]
}
```

## Relations
- **MemberOf**: repo-automation-decision-1739116785000
- **DependsOn**: []

## Advantages
✅ **Zero Maintenance**: Code Climate handles all automation
✅ **Professional Dashboards**: Beautiful trend visualization
✅ **Industry Standard**: Battle-tested, proven solution
✅ **Comprehensive**: Coverage, complexity, security, duplication
✅ **GitHub Integration**: Native PR comments, status checks

## Disadvantages
❌ **Cost**: Free tier only for open source ($15+/month for private repos)
❌ **Vendor Lock-in**: Hard to migrate away from Code Climate
❌ **External Dependency**: Service outage blocks CI
❌ **Limited Customization**: Must work within Code Climate's constraints
❌ **Data Outside Repo**: Quality data stored on external servers

## Dependencies
None (external SaaS service)

## Metadata
- **Author**: FPF Phase 1 (Abduction)
- **Category**: Minimalist (least effort, highest service)
- **Complexity**: Low (setup only)
- **Risk**: Medium (vendor lock-in, cost)
- **Estimated Runtime Impact**: +2-3 minutes to CI
- **Estimated Cost**: $0 (open source) or $15-50/month (private)
- **Maintenance Burden**: None (SaaS)
- **Vendor Lock-in**: High (quality data stored externally)
