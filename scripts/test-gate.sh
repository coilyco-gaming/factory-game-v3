#!/usr/bin/env bash

set -euo pipefail

project="tests.csproj"
restore_timeout="${WARD_TEST_RESTORE_TIMEOUT:-4m}"
test_timeout="${WARD_TEST_TEST_TIMEOUT:-6m}"

if ! command -v dotnet >/dev/null 2>&1; then
  echo "ward: dotnet not found on PATH" >&2
  exit 127
fi

echo "ward: restore ${project}"
timeout --kill-after=30s "${restore_timeout}" dotnet restore "${project}" --nologo --verbosity minimal --disable-parallel

echo "ward: test ${project}"
timeout --kill-after=30s "${test_timeout}" dotnet test "${project}" --no-restore --nologo --verbosity normal
