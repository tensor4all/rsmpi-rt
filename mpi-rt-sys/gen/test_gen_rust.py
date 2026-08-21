import pathlib
import re
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).parent))

import gen_rust


class GeneratorTests(unittest.TestCase):
    def test_constants_use_data_symbol_helper(self):
        generated = gen_rust.generate_constants()
        mapped = [
            (c_type, name)
            for c_type, name in gen_rust.constants
            if c_type in gen_rust.CONST_TYPE_MAP
        ]

        self.assertNotIn("lib.get::<", generated)
        self.assertEqual(generated.count("loader::get_constant::<"), len(mapped) + 2)
        for c_type, name in mapped:
            rust_type, _ = gen_rust.CONST_TYPE_MAP[c_type]
            symbol = gen_rust.mpi_to_mpiabi_const(name)
            self.assertIn(
                f'{name.lower()}: loader::get_constant::<{rust_type}>(b"{symbol}\\0")',
                generated,
            )

        for c_type, name in (
            ("int", "MPI_ANY_SOURCE"),
            ("MPI_Comm", "MPI_COMM_WORLD"),
            ("void *", "MPI_BOTTOM"),
            ("MPI_Comm_copy_attr_function *", "MPI_COMM_DUP_FN"),
        ):
            rust_type, _ = gen_rust.CONST_TYPE_MAP[c_type]
            symbol = gen_rust.mpi_to_mpiabi_const(name)
            self.assertRegex(
                generated,
                re.escape(
                    f'{name.lower()}: loader::get_constant::<{rust_type}>(b"{symbol}\\0")'
                ),
            )

    def test_function_lookups_use_mpiabi_symbols(self):
        generated = gen_rust.generate_functions()
        lookups = re.findall(r'loader::get_symbol::<F>\(b"([^\"]+)"\)', generated)

        expected = [gen_rust.mpi_to_mpiabi_func(name) for _, name, _, _ in gen_rust.functions]
        self.assertEqual(lookups, [f"{symbol}\\0" for symbol in expected])
        self.assertTrue(all(name.startswith(("MPIABI_", "MPIXABI_")) for name in expected))
        for symbol in expected:
            self.assertIn(f'loader::get_symbol::<F>(b"{symbol}\\0")', generated)

    def test_function_symbol_helper_rejects_non_mpi_names(self):
        with self.assertRaisesRegex(AssertionError, "BAD_Init"):
            gen_rust.mpi_to_mpiabi_func("BAD_Init")

    def test_loader_helper_reads_data_symbol_value(self):
        loader = (pathlib.Path(__file__).parent.parent / "src" / "loader.rs").read_text()
        self.assertIn("pub(crate) unsafe fn get_constant<T: Copy>", loader)
        self.assertIn("libloading::Symbol<*const T>", loader)
        self.assertIn("**sym", loader)


if __name__ == "__main__":
    unittest.main()
