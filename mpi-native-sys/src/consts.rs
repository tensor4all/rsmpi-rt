//! Per-family predefined constants (docs/design/mpi-native-abi.md §2 and §4).
//!
//! MPICH ABI and standard ABI: integer literals from the family's `mpi.h`. Open MPI: the
//! **addresses** of exported `ompi_*` data symbols, read once. Sources: MPICH 5.0.2 and
//! Intel MPI 2021.19 headers; Open MPI 4.1.6 `mpi.h.in` and binary; `mpi-forum/mpi-abi-stubs`
//! `d224b2bf` `mpi.h`.

use std::os::raw::c_int;

use crate::detect::Family;

/// The constants the carried function set and rsmpi-rt's communicator path read.
#[derive(Clone, Debug)]
pub struct Consts {
    pub comm_world: usize,
    pub comm_self: usize,
    pub comm_null: usize,
    pub datatype_null: usize,
    pub info_null: usize,
    pub message_null: usize,
    pub message_no_proc: usize,
    pub request_null: usize,
    pub group_null: usize,
    pub group_empty: usize,
    pub errors_are_fatal: usize,
    pub errors_return: usize,
    /// `MPI_STATUS_IGNORE` as the family's C library expects it (address value).
    pub status_ignore: usize,
    pub any_source: c_int,
    pub any_tag: c_int,
    pub proc_null: c_int,
    pub undefined: c_int,
    pub thread_single: c_int,
    pub thread_funneled: c_int,
    pub thread_serialized: c_int,
    pub thread_multiple: c_int,
    pub tag_ub: c_int,
    pub appnum: c_int,
    pub universe_size: c_int,
    pub ident: c_int,
    pub congruent: c_int,
    pub similar: c_int,
    pub unequal: c_int,
    pub cart: c_int,
    pub graph: c_int,
    pub dist_graph: c_int,
    pub comm_type_shared: c_int,
    pub max_processor_name: c_int,
    pub max_library_version_string: c_int,
    pub byte: usize,
    pub int8_t: usize,
    pub int16_t: usize,
    pub int32_t: usize,
    pub int64_t: usize,
    pub uint8_t: usize,
    pub uint16_t: usize,
    pub uint32_t: usize,
    pub uint64_t: usize,
    pub float: usize,
    pub double: usize,
    pub c_bool: usize,
    /// Offsets of `MPI_SOURCE`, `MPI_TAG`, `MPI_ERROR` in the family's `MPI_Status`.
    pub status_offsets: (usize, usize, usize),
}

/// `MPI_SUCCESS` is 0 in every family.
pub const SUCCESS: c_int = 0;

/// Smaller than the largest family's value, so a buffer of this size is always enough:
/// 8192 for the version string, 1024 for the error string (twice the 512 of MPICH and the
/// standard ABI).
pub const VERSION_BUFFER: usize = 8192;
/// See [`VERSION_BUFFER`].
pub const ERROR_BUFFER: usize = 1024;

impl Consts {
    /// The table of `family`. `addr` resolves an exported data symbol to its address; it
    /// is called only for [`Family::OpenMpi`].
    pub fn for_family(family: Family, mut addr: impl FnMut(&str) -> usize) -> Consts {
        match family {
            Family::Mpich => Consts {
                comm_world: 0x4400_0000,
                comm_self: 0x4400_0001,
                comm_null: 0x0400_0000,
                datatype_null: 0x0c00_0000,
                info_null: 0x1c00_0000,
                message_null: 0x2c00_0000,
                message_no_proc: 0x6c00_0000,
                request_null: 0x2c00_0000,
                group_null: 0x0800_0000,
                group_empty: 0x4800_0000,
                errors_are_fatal: 0x5400_0000,
                errors_return: 0x5400_0001,
                status_ignore: 1,
                any_source: -2,
                any_tag: -1,
                proc_null: -1,
                undefined: -32766,
                thread_single: 0,
                thread_funneled: 1,
                thread_serialized: 2,
                thread_multiple: 3,
                tag_ub: 0x6440_0001,
                appnum: 0x6440_000d,
                universe_size: 0x6440_0009,
                ident: 0,
                congruent: 1,
                similar: 2,
                unequal: 3,
                graph: 1,
                cart: 2,
                dist_graph: 3,
                comm_type_shared: 1,
                max_processor_name: 128,
                max_library_version_string: 8192,
                byte: 0x4c00_010d,
                int8_t: 0x4c00_0137,
                int16_t: 0x4c00_0238,
                int32_t: 0x4c00_0439,
                int64_t: 0x4c00_083a,
                uint8_t: 0x4c00_013b,
                uint16_t: 0x4c00_023c,
                uint32_t: 0x4c00_043d,
                uint64_t: 0x4c00_083e,
                float: 0x4c00_040a,
                double: 0x4c00_080b,
                c_bool: 0x4c00_013f,
                status_offsets: (8, 12, 16),
            },
            Family::StandardAbi => Consts {
                comm_world: 0x101,
                comm_self: 0x102,
                comm_null: 0x100,
                datatype_null: 0x200,
                info_null: 0x130,
                message_null: 0x128,
                message_no_proc: 0x129,
                request_null: 0x180,
                group_null: 0x108,
                group_empty: 0x109,
                errors_are_fatal: 0x141,
                errors_return: 0x143,
                status_ignore: 0,
                any_source: -1,
                any_tag: -2,
                proc_null: -3,
                undefined: -32766,
                thread_single: 0,
                thread_funneled: 1024,
                thread_serialized: 2048,
                thread_multiple: 4096,
                tag_ub: 501,
                appnum: 505,
                universe_size: 507,
                ident: 201,
                congruent: 202,
                similar: 203,
                unequal: 204,
                cart: 211,
                graph: 212,
                dist_graph: 213,
                comm_type_shared: 221,
                max_processor_name: 256,
                max_library_version_string: 8192,
                byte: 0x247,
                int8_t: 0x240,
                int16_t: 0x248,
                int32_t: 0x250,
                int64_t: 0x258,
                uint8_t: 0x241,
                uint16_t: 0x249,
                uint32_t: 0x251,
                uint64_t: 0x259,
                float: 0x210,
                double: 0x214,
                c_bool: 0x238,
                status_offsets: (0, 4, 8),
            },
            Family::OpenMpi => Consts {
                comm_world: addr("ompi_mpi_comm_world"),
                comm_self: addr("ompi_mpi_comm_self"),
                comm_null: addr("ompi_mpi_comm_null"),
                datatype_null: addr("ompi_mpi_datatype_null"),
                info_null: addr("ompi_mpi_info_null"),
                message_null: addr("ompi_message_null"),
                message_no_proc: addr("ompi_message_no_proc"),
                request_null: addr("ompi_request_null"),
                group_null: addr("ompi_mpi_group_null"),
                group_empty: addr("ompi_mpi_group_empty"),
                errors_are_fatal: addr("ompi_mpi_errors_are_fatal"),
                errors_return: addr("ompi_mpi_errors_return"),
                status_ignore: 0,
                any_source: -1,
                any_tag: -1,
                proc_null: -2,
                undefined: -32766,
                thread_single: 0,
                thread_funneled: 1,
                thread_serialized: 2,
                thread_multiple: 3,
                // Attribute keys are the enum order of `mpi.h.in`: TAG_UB, HOST, IO,
                // WTIME_IS_GLOBAL, APPNUM, LASTUSEDCODE, UNIVERSE_SIZE.
                tag_ub: 0,
                appnum: 4,
                universe_size: 6,
                ident: 0,
                congruent: 1,
                similar: 2,
                unequal: 3,
                cart: 1,
                graph: 2,
                dist_graph: 3,
                comm_type_shared: 0,
                max_processor_name: 256,
                max_library_version_string: 256,
                byte: addr("ompi_mpi_byte"),
                int8_t: addr("ompi_mpi_int8_t"),
                int16_t: addr("ompi_mpi_int16_t"),
                int32_t: addr("ompi_mpi_int32_t"),
                int64_t: addr("ompi_mpi_int64_t"),
                uint8_t: addr("ompi_mpi_uint8_t"),
                uint16_t: addr("ompi_mpi_uint16_t"),
                uint32_t: addr("ompi_mpi_uint32_t"),
                uint64_t: addr("ompi_mpi_uint64_t"),
                float: addr("ompi_mpi_float"),
                double: addr("ompi_mpi_double"),
                c_bool: addr("ompi_mpi_c_bool"),
                status_offsets: (0, 4, 8),
            },
        }
    }

    /// Whether `handle` may be passed to MPI as a communicator: not `0`, not the family's
    /// `MPI_COMM_NULL`. This is a check on the *encoding*, not an identity check (D4).
    pub fn comm_handle_is_plausible(&self, handle: usize) -> bool {
        handle != 0 && handle != self.comm_null
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_are_consistent_with_the_design() {
        let m = Consts::for_family(Family::Mpich, |_| unreachable!());
        let a = Consts::for_family(Family::StandardAbi, |_| unreachable!());
        assert_eq!(
            (m.comm_world, m.comm_self, m.comm_null),
            (0x44000000, 0x44000001, 0x04000000)
        );
        assert_eq!(
            (a.comm_world, a.comm_self, a.comm_null),
            (0x101, 0x102, 0x100)
        );
        assert_eq!(
            (m.byte, m.uint8_t, a.byte, a.uint8_t),
            (0x4c00010d, 0x4c00013b, 0x247, 0x241)
        );
        assert_eq!(
            (m.any_source, m.proc_null, a.any_source, a.proc_null),
            (-2, -1, -1, -3)
        );
        assert_eq!((m.status_offsets.0, a.status_offsets.0), (8, 0));
        assert_eq!((a.thread_serialized, a.thread_multiple), (2048, 4096));
        assert!(m.thread_single < m.thread_funneled && a.thread_funneled < a.thread_serialized);
    }

    #[test]
    fn open_mpi_constants_are_addresses_read_once() {
        let mut seen = Vec::new();
        let o = Consts::for_family(Family::OpenMpi, |n| {
            seen.push(n.to_string());
            0x1000 + seen.len()
        });
        assert!(seen.contains(&"ompi_mpi_comm_world".to_string()));
        assert_ne!(o.comm_world, o.comm_self);
        assert_eq!((o.tag_ub, o.status_ignore, o.proc_null), (0, 0, -2));
    }

    #[test]
    fn null_and_zero_handles_are_not_plausible() {
        let m = Consts::for_family(Family::Mpich, |_| unreachable!());
        assert!(!m.comm_handle_is_plausible(0));
        assert!(!m.comm_handle_is_plausible(m.comm_null));
        assert!(m.comm_handle_is_plausible(m.comm_world));
    }
}
