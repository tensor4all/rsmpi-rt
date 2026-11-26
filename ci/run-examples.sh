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
result="ok"

for example in ${examples}
do
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

printf "\nexample result: ${result}. ${num_ok} passed; ${num_failed} failed\n\n"
exit ${num_failed}
