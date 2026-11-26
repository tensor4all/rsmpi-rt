#!/bin/sh

set -e

# enable oversubscribing for Open-MPI 4.x
export OMPI_MCA_rmaps_base_oversubscribe=1
# Open-MPI 5.x: https://github.com/open-mpi/ompi/issues/8955#issuecomment-846272943
export PRTE_MCA_rmaps_default_mapping_policy=:oversubscribe

EXAMPLES_DIR="examples"

examples=$(ls ${EXAMPLES_DIR} | sed "s/\\.rs\$//")
num_examples=$(printf "%d" "$(echo "${examples}" | wc -w)")

maxnp=3
printf "running %d examples\n" ${num_examples}

num_ok=0
num_failed=0
num_skipped=0
result="ok"

# Open MPI major version, empty for MPICH, Intel MPI and the rest. `mpiexec --version` is not used:
# under Open MPI 5.x on the hosted runner it prints only a missing-help-file message (seen in the
# diagnostics of rsmpi-rt#8), so ask `ompi_info`, and fall back to the launcher itself: in Open MPI 5.x
# `mpiexec` is a symlink to `prterun`, in 4.x to `orterun`.
ompi_desc=""
ompi_major=$(ompi_info --parsable 2>/dev/null | sed -n 's/^ompi:version:full:\([0-9][0-9]*\)\..*/\1/p' | head -n1)
if [ -n "${ompi_major}" ]; then
  ompi_desc="Open MPI $(ompi_info --parsable 2>/dev/null | sed -n 's/^ompi:version:full://p' | head -n1)"
else
  case "$(basename "$(readlink -f "$(command -v mpiexec)" 2>/dev/null)" 2>/dev/null)" in
    prterun|prte) ompi_major=5; ompi_desc="Open MPI 5.x (mpiexec is prterun)" ;;
  esac
fi

# Only `spawn_multiple` is gated, and only here. A version test and not a probe: a probe would be
# the example itself, and treating its crash as a skip would hide a real regression. Open MPI 5.x's
# PRRTE segfaults in MPI_Comm_spawn_multiple on the hosted runner (diagnostics in
# https://github.com/tensor4all/rsmpi-rt/pull/8), while 4.1.6 passes it. Whether the child failing to
# find the library the parent loaded is part of the cause is NOT established, so this is a known
# unknown, not a settled limitation. A later 5.x point release that fixes it stays skipped until
# this line is changed on purpose.
skip_reason() {
  if [ "$1" = "spawn_multiple" ] && [ -n "${ompi_major}" ] && [ "${ompi_major}" -ge 5 ]; then
    printf "MPI_Comm_spawn_multiple unsupported on %s: it segfaults under Open MPI 5.x's PRRTE (see rsmpi-rt#8)" \
      "${ompi_desc}"
    return 0
  fi
  return 1
}

for example in ${examples}
do
  if reason=$(skip_reason "${example}"); then
    printf "example ${example} SKIPPED: %s\n" "${reason}"
    num_skipped=$((${num_skipped} + 1))
    continue
  fi
  printf "example ${example} on 2...${maxnp} processes"
  output_file="/tmp/${example}_output"
  for num_proc in $(seq 2 ${maxnp})
  do
    if (cargo mpirun "$@" --verbose -n ${num_proc} --example "${example}" > "${output_file}" 2>&1)
    then
      printf "."
      rm -f "${output_file}"
    else
      status=$?
      printf " failed on %d processes (cargo mpirun exit status %d).\noutput:\n" ${num_proc} ${status}
      cat "${output_file}"
      # `cargo mpirun` can fail without any output from the program itself (an example that
      # spawns children reports nothing if mpiexec dies first). Run the built binary directly,
      # with the launcher's own diagnostics on, so the log says why. This is diagnostics only:
      # the example's result above still counts as failed.
      binary="target/debug/examples/${example}"
      if [ -x "${binary}" ]; then
        printf -- "--- diagnostics: mpiexec -n %d %s\n" ${num_proc} "${binary}"
        mpiexec --version 2>&1 | head -n 2 || true
        (mpiexec --display-map --debug-daemons -n ${num_proc} "${binary}" 2>&1 ||
          mpiexec -n ${num_proc} "${binary}" 2>&1) | head -n 80
        printf -- "--- diagnostics: exit status %d\n" $?
      fi
      rm -f "${output_file}"
      num_failed=$((${num_failed} + 1))
      result="failed"
      continue 2
    fi
  done
  printf " ok.\n"
  num_ok=$((${num_ok} + 1))
done

printf "\nexample result: ${result}. ${num_ok} passed; ${num_failed} failed; ${num_skipped} skipped\n\n"
exit ${num_failed}
