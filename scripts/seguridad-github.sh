#!/usr/bin/env bash
# Lo abierto en la pestaña Security de GitHub (ADR 0034): avisos de CodeQL y Scorecard, alertas de
# Dependabot, secretos subidos por error y avisos privados de vulnerabilidades. Solo lee.
#
# Se repasa al empezar cada sesión y después de cada subida, cuando acaba la CI (CLAUDE.md):
#   scripts/seguridad-github.sh
# Lo que ya se revisó y se aceptó se cierra en GitHub con su motivo, y queda en docs/deuda.md.

set -euo pipefail
REPO=webcafeina/Apolo

seccion() { printf '\n== %s ==\n' "$1"; }

seccion "Code scanning (CodeQL y Scorecard)"
gh api --paginate "repos/$REPO/code-scanning/alerts?state=open&per_page=100" \
  --jq '.[] | "#\(.number) [\(.tool.name) · \(.rule.security_severity_level // .rule.severity)] \(.rule.description) · \(.most_recent_instance.location.path // "-"):\(.most_recent_instance.location.start_line // "")"' |
  sort -t'#' -k2 -n || echo "(no se pudo leer)"

seccion "Dependabot"
gh api --paginate "repos/$REPO/dependabot/alerts?state=open&per_page=100" \
  --jq '.[] | "#\(.number) [\(.security_advisory.severity)] \(.dependency.package.name) (\(.dependency.package.ecosystem)): \(.security_advisory.summary) · arreglo: \(.security_vulnerability.first_patched_version.identifier // "ninguno")"' ||
  echo "(no se pudo leer)"

seccion "Secretos subidos por error"
gh api --paginate "repos/$REPO/secret-scanning/alerts?state=open&per_page=100" \
  --jq '.[] | "#\(.number) \(.secret_type_display_name) · desde \(.created_at[0:10])"' || echo "(no se pudo leer)"

seccion "Avisos privados de vulnerabilidades"
gh api "repos/$REPO/security-advisories?state=triage" --jq '.[] | "\(.ghsa_id) [\(.severity)] \(.summary)"' ||
  echo "(no se pudo leer)"

seccion "PR abiertos"
gh pr list --repo "$REPO" --json number,title,author -q '.[] | "#\(.number) \(.title) · \(.author.login)"'
