#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
ROOT_DIR="$(cd "${RUST_DIR}/.." && pwd)"

POLICY_JSON="${PGEN_BRANCH_PROTECTION_POLICY_JSON:-${RUST_DIR}/config/branch_protection_policy.json}"
WORKFLOWS_DIR="${PGEN_BRANCH_PROTECTION_WORKFLOWS_DIR:-${ROOT_DIR}/.github/workflows}"
STATE_DIR="${PGEN_BRANCH_PROTECTION_STATE_DIR:-${RUST_DIR}/target/branch_protection_contract_gate}"
WORK_DIR="${STATE_DIR}/work"

EXPECTED_MINIMUM_CHECKS=(
  "sota-exit-gate"
  "annotation-contract-gate"
  "differential-regression-gate"
  "fixed-point-gate"
  "performance-gate"
)

fail() {
  echo "error: $*" >&2
  exit 1
}

write_list_file() {
  local output_file="$1"
  shift || true
  : > "${output_file}"
  if [[ "$#" -gt 0 ]]; then
    printf "%s\n" "$@" > "${output_file}"
  fi
}

mkdir -p "${WORK_DIR}"
rm -f "${WORK_DIR}/"*.txt "${WORK_DIR}/"*.env "${WORK_DIR}/"*.json "${WORK_DIR}/"*.raw "${WORK_DIR}/"*.tsv

[[ -f "${POLICY_JSON}" ]] || fail "branch protection policy file not found at '${POLICY_JSON}'"
[[ -d "${WORKFLOWS_DIR}" ]] || fail "workflow directory not found at '${WORKFLOWS_DIR}'"

REQUIRED_CHECKS_FILE="${WORK_DIR}/required_status_checks.txt"
POLICY_METADATA_ENV="${WORK_DIR}/policy_metadata.env"

perl -MJSON::PP -e '
  use strict;
  use warnings;

  my ($policy_path, $required_checks_path, $metadata_env_path) = @ARGV;
  local $/;

  open my $fh, "<", $policy_path or die "open $policy_path: $!";
  my $data = decode_json(<$fh>);
  close $fh;

  die "policy root must be an object\n" unless ref($data) eq "HASH";

  for my $required_key (qw(version default_branch require_up_to_date_before_merge required_status_checks)) {
    die "missing required key: $required_key\n" unless exists $data->{$required_key};
  }

  die "version must be an integer >= 1\n"
    unless defined($data->{version}) && !ref($data->{version}) && $data->{version} =~ /^\d+$/ && $data->{version} >= 1;
  die "default_branch must be a non-empty string\n"
    unless defined($data->{default_branch}) && !ref($data->{default_branch}) && $data->{default_branch} ne q{};

  my $up_to_date = $data->{require_up_to_date_before_merge};
  my $is_bool = ref($up_to_date) eq "JSON::PP::Boolean" || (!ref($up_to_date) && ($up_to_date eq "0" || $up_to_date eq "1"));
  die "require_up_to_date_before_merge must be a JSON boolean\n" unless $is_bool;

  my $checks = $data->{required_status_checks};
  die "required_status_checks must be a non-empty array\n" unless ref($checks) eq "ARRAY" && @$checks > 0;

  my %seen;
  open my $checks_fh, ">", $required_checks_path or die "open $required_checks_path: $!";
  for my $check (@$checks) {
    die "required_status_checks entries must be non-empty strings\n"
      unless defined($check) && !ref($check) && $check ne q{};
    die "duplicate required_status_checks entry: $check\n" if $seen{$check}++;
    print {$checks_fh} "$check\n";
  }
  close $checks_fh;

  open my $meta_fh, ">", $metadata_env_path or die "open $metadata_env_path: $!";
  print {$meta_fh} "policy_version=$data->{version}\n";
  print {$meta_fh} "default_branch=$data->{default_branch}\n";
  print {$meta_fh} "require_up_to_date_before_merge=", ($up_to_date ? "true" : "false"), "\n";
  print {$meta_fh} "required_status_checks_count=", scalar(@$checks), "\n";
  close $meta_fh;
' "${POLICY_JSON}" "${REQUIRED_CHECKS_FILE}" "${POLICY_METADATA_ENV}"

source "${POLICY_METADATA_ENV}"

mapfile -t REQUIRED_CHECKS < "${REQUIRED_CHECKS_FILE}"
[[ "${#REQUIRED_CHECKS[@]}" -gt 0 ]] || fail "branch protection policy declares zero required checks"

AVAILABLE_CHECKS_RAW="${WORK_DIR}/available_status_checks.raw"
AVAILABLE_CHECKS_FILE="${WORK_DIR}/available_status_checks.txt"
CHECK_SOURCE_TSV="${WORK_DIR}/available_check_sources.tsv"
: > "${AVAILABLE_CHECKS_RAW}"
: > "${CHECK_SOURCE_TSV}"

shopt -s nullglob
WORKFLOW_FILES=("${WORKFLOWS_DIR}"/*.yml)
shopt -u nullglob
[[ "${#WORKFLOW_FILES[@]}" -gt 0 ]] || fail "no workflow files found in '${WORKFLOWS_DIR}'"

for workflow_file in "${WORKFLOW_FILES[@]}"; do
  workflow_checks="$(
    perl -ne '
      if (/^name:\s*(.+?)\s*$/) {
        print "$1\n";
      } elsif (/^ {4}name:\s*(.+?)\s*$/) {
        print "$1\n";
      }
    ' "${workflow_file}"
  )"

  [[ -n "${workflow_checks}" ]] || fail "workflow file '${workflow_file}' is missing a workflow/job name"

  while IFS= read -r workflow_check; do
    [[ -n "${workflow_check}" ]] || continue
    printf "%s\n" "${workflow_check}" >> "${AVAILABLE_CHECKS_RAW}"
    printf "%s\t%s\n" "${workflow_check}" "${workflow_file}" >> "${CHECK_SOURCE_TSV}"
  done <<< "${workflow_checks}"
done

sort -u "${AVAILABLE_CHECKS_RAW}" > "${AVAILABLE_CHECKS_FILE}"

MISSING_MINIMUM_CHECKS=()
for check in "${EXPECTED_MINIMUM_CHECKS[@]}"; do
  if ! grep -Fxq "${check}" "${REQUIRED_CHECKS_FILE}"; then
    MISSING_MINIMUM_CHECKS+=("${check}")
  fi
done

POLICY_CHECKS_MISSING_WORKFLOW=()
POLICY_CHECKS_MISSING_PR_TRIGGER=()

for check in "${REQUIRED_CHECKS[@]}"; do
  if ! grep -Fxq "${check}" "${AVAILABLE_CHECKS_FILE}"; then
    POLICY_CHECKS_MISSING_WORKFLOW+=("${check}")
    continue
  fi

  workflow_file="$(
    awk -F '\t' -v wanted="${check}" '$1 == wanted { print $2; exit }' "${CHECK_SOURCE_TSV}"
  )"
  [[ -n "${workflow_file}" ]] || fail "failed to resolve workflow source for check '${check}'"

  if ! rg -q '^  pull_request:' "${workflow_file}"; then
    POLICY_CHECKS_MISSING_PR_TRIGGER+=("${check}")
  fi
done

write_list_file "${WORK_DIR}/missing_minimum_checks.txt" "${MISSING_MINIMUM_CHECKS[@]}"
write_list_file "${WORK_DIR}/policy_checks_missing_workflow.txt" "${POLICY_CHECKS_MISSING_WORKFLOW[@]}"
write_list_file "${WORK_DIR}/policy_checks_missing_pull_request_trigger.txt" "${POLICY_CHECKS_MISSING_PR_TRIGGER[@]}"

REPORT_JSON="${STATE_DIR}/report.json"
SUMMARY_TXT="${STATE_DIR}/summary.txt"

mkdir -p "${STATE_DIR}"

perl -MJSON::PP -e '
  use strict;
  use warnings;

  sub read_lines {
    my ($path) = @_;
    open my $fh, "<", $path or die "open $path: $!";
    my @lines = grep { length $_ } map { chomp; $_ } <$fh>;
    close $fh;
    return \@lines;
  }

  my ($policy_path, $metadata_env_path, $available_checks_path, $missing_minimum_path, $missing_workflow_path, $missing_pr_path, $output_path) = @ARGV;

  open my $policy_fh, "<", $policy_path or die "open $policy_path: $!";
  my $policy_json = do { local $/; <$policy_fh> };
  my $policy = decode_json($policy_json);
  close $policy_fh;

  open my $meta_fh, "<", $metadata_env_path or die "open $metadata_env_path: $!";
  my %meta;
  while (my $line = <$meta_fh>) {
    chomp $line;
    next if $line eq q{};
    my ($key, $value) = split /=/, $line, 2;
    $meta{$key} = $value;
  }
  close $meta_fh;

  my $report = {
    result => (
      @{read_lines($missing_minimum_path)} == 0 &&
      @{read_lines($missing_workflow_path)} == 0 &&
      @{read_lines($missing_pr_path)} == 0
    ) ? "pass" : "fail",
    policy_file => $policy_path,
    policy_version => 0 + $meta{policy_version},
    default_branch => $meta{default_branch},
    require_up_to_date_before_merge => ($meta{require_up_to_date_before_merge} eq "true" ? JSON::PP::true : JSON::PP::false),
    required_status_checks => $policy->{required_status_checks},
    available_workflow_checks => read_lines($available_checks_path),
    missing_minimum_checks => read_lines($missing_minimum_path),
    policy_checks_missing_workflow => read_lines($missing_workflow_path),
    policy_checks_missing_pull_request_trigger => read_lines($missing_pr_path),
  };

  open my $out_fh, ">", $output_path or die "open $output_path: $!";
  print {$out_fh} JSON::PP->new->ascii->pretty->canonical->encode($report);
  close $out_fh;
' \
  "${POLICY_JSON}" \
  "${POLICY_METADATA_ENV}" \
  "${AVAILABLE_CHECKS_FILE}" \
  "${WORK_DIR}/missing_minimum_checks.txt" \
  "${WORK_DIR}/policy_checks_missing_workflow.txt" \
  "${WORK_DIR}/policy_checks_missing_pull_request_trigger.txt" \
  "${REPORT_JSON}"

{
  echo "Branch protection contract summary"
  echo "Policy file: ${POLICY_JSON}"
  echo "Policy version: ${policy_version}"
  echo "Default branch: ${default_branch}"
  echo "Require up-to-date before merge: ${require_up_to_date_before_merge}"
  echo "Required status checks (${#REQUIRED_CHECKS[@]}):"
  for check in "${REQUIRED_CHECKS[@]}"; do
    echo "- ${check}"
  done
  echo "Available workflow/job checks: $(wc -l < "${AVAILABLE_CHECKS_FILE}" | tr -d " ")"
  echo "Missing minimum checks: ${#MISSING_MINIMUM_CHECKS[@]}"
  echo "Policy checks missing workflow: ${#POLICY_CHECKS_MISSING_WORKFLOW[@]}"
  echo "Policy checks missing pull_request trigger: ${#POLICY_CHECKS_MISSING_PR_TRIGGER[@]}"
} > "${SUMMARY_TXT}"

if [[ "${#MISSING_MINIMUM_CHECKS[@]}" -gt 0 ]]; then
  fail "policy is missing roadmap/release minimum checks: ${MISSING_MINIMUM_CHECKS[*]}"
fi

if [[ "${#POLICY_CHECKS_MISSING_WORKFLOW[@]}" -gt 0 ]]; then
  fail "policy checks are not backed by workflow/job names: ${POLICY_CHECKS_MISSING_WORKFLOW[*]}"
fi

if [[ "${#POLICY_CHECKS_MISSING_PR_TRIGGER[@]}" -gt 0 ]]; then
  fail "policy checks do not run on pull_request: ${POLICY_CHECKS_MISSING_PR_TRIGGER[*]}"
fi

echo "✅ Branch protection contract gate passed."
echo "   Summary: ${SUMMARY_TXT}"
echo "   Report:  ${REPORT_JSON}"
