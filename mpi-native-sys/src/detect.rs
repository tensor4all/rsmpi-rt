//! Family detection: the pure part.
//!
//! Nothing here touches a library. [`classify_version`] is the version-string table of
//! docs/design/mpi-native-abi.md §1 (tenferro-rs), and [`decide`] is the detection order
//! D3 applied to a list of [`Candidate`]s that the loader built from the loaded objects.
//! Keeping it pure is what lets the order be unit-tested with mocked object lists.

use std::fmt;

/// A handle/constant family this crate can speak.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Family {
    /// MPICH ABI: 32-bit `int` handles (MPICH, Intel MPI, MVAPICH, Cray MPICH, HPE HMPT).
    Mpich,
    /// Open MPI ABI: pointer handles to exported `ompi_mpi_*` data symbols.
    OpenMpi,
    /// MPI 5.0 standard ABI v1: pointer-width tokens with fixed integer values.
    StandardAbi,
}

impl Family {
    /// Short stable name, used in `tfe_mpi_library_info`.
    pub fn name(self) -> &'static str {
        match self {
            Family::Mpich => "mpich-abi",
            Family::OpenMpi => "openmpi-abi",
            Family::StandardAbi => "standard-abi",
        }
    }
}

/// Why a version string is not a supported family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reject {
    /// MPIwrapper (MPItrampoline's backend): removed in tenferro-rs#2735.
    MpiWrapper,
    /// Microsoft MPI: Linux and macOS only.
    MsMpi,
    /// HPE MPT: its own ABI (`MPI_Comm = unsigned`), not MPICH-compatible.
    HpeMpt,
    /// Anything the table does not list, or a listed one below its version floor.
    Unknown,
}

/// Result of classifying one `MPI_Get_library_version` string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    /// A supported family.
    Family(Family),
    /// A named rejection.
    Reject(Reject),
}

/// Parse the leading `major.minor` of a dotted version, skipping leading separators.
fn dotted(s: &str) -> Option<(u32, u32)> {
    let s = s.trim_start_matches(|c: char| !c.is_ascii_digit());
    let mut parts = s.split(|c: char| !c.is_ascii_digit());
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    Some((major, minor))
}

fn after<'a>(s: &'a str, marker: &str) -> Option<&'a str> {
    s.find(marker).map(|i| &s[i + marker.len()..])
}

/// Classify a version string. The order of the rows is part of the contract: the
/// standard-ABI MPICH row precedes plain `MPICH`, and `MPIwrapper` comes first.
///
/// ```
/// use mpi_native_sys::detect::{classify_version, Family, Row};
/// let cray = "MPI VERSION    : CRAY MPICH version 7.7.10 (ANL base 3.2)";
/// assert_eq!(classify_version(cray), Row::Family(Family::Mpich));
/// ```
pub fn classify_version(s: &str) -> Row {
    use Family::*;
    let s = s.trim_start();
    let fam = Row::Family;
    let rej = Row::Reject;
    if s.starts_with("MPIwrapper") {
        return rej(Reject::MpiWrapper);
    }
    if s.starts_with("MPICH") && s.contains("--enable-mpi-abi") {
        return fam(StandardAbi);
    }
    if s.starts_with("mpi_abi_wrapper") {
        return fam(StandardAbi);
    }
    if s.starts_with("MPICH") {
        return match after(s, "MPICH Version:").and_then(dotted) {
            Some(v) if v >= (3, 1) => fam(Mpich),
            _ => rej(Reject::Unknown),
        };
    }
    if s.starts_with("Open MPI") {
        return fam(OpenMpi);
    }
    if s.starts_with("Microsoft MPI") {
        return rej(Reject::MsMpi);
    }
    if s.starts_with("Intel") {
        // "Intel(R) MPI Library 2021.19.0 ..." and "... 2018 Update 3": the year, > 2014.
        let year = after(s, "Library").and_then(|r| dotted(r)).map(|v| v.0);
        return match year {
            Some(y) if y > 2014 => fam(Mpich),
            _ => rej(Reject::Unknown),
        };
    }
    if s.starts_with("MVAPICH") {
        return match after(s, "Version").and_then(dotted) {
            Some(v) if v.0 >= 2 => fam(Mpich),
            _ => rej(Reject::Unknown),
        };
    }
    if let Some(rest) = after(s, "CRAY MPICH") {
        // A substring row: the real string is "MPI VERSION    : CRAY MPICH version 7.7.10".
        return match dotted(rest) {
            Some(v) if v.0 >= 7 => fam(Mpich),
            _ => rej(Reject::Unknown),
        };
    }
    if s.starts_with("FUJITSU MPI") {
        return fam(OpenMpi);
    }
    if s.starts_with("HPE MPT") {
        return rej(Reject::HpeMpt);
    }
    if s.starts_with("HPE HMPT") {
        return fam(Mpich);
    }
    rej(Reject::Unknown)
}

/// What the loader learned about one loaded object. Built from `dlsym` and calls, but
/// consumed here as plain data.
#[derive(Clone, Debug, Default)]
pub struct Candidate {
    /// Path of the object as the dynamic linker reports it.
    pub path: String,
    /// Defines `MPIABI_Get_library_version` (MPItrampoline / MPIwrapper).
    pub mpiabi_marker: bool,
    /// Defines `PMPI_Get_library_version` (a profiling interposer defines only `MPI_*`).
    pub pmpi_version: bool,
    /// The `abi_major` that `MPI_Abi_get_version` returned, when the object defines it
    /// and the call succeeded. Native builds return `-1`.
    pub abi_major: Option<i32>,
    /// The `abi_minor` returned alongside `abi_major`.
    pub abi_minor: Option<i32>,
    /// `MPI_Get_library_version`, when the object is an MPI candidate.
    pub version: Option<String>,
    /// Defines the data symbol `ompi_mpi_comm_world`.
    pub ompi_witness: bool,
    /// Defines the function `MPIR_Err_return_comm`.
    pub mpich_witness: bool,
}

/// A detection failure. `Display` renders the layout of the design's error text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    /// The host uses MPItrampoline/MPIwrapper.
    MpiTrampolineNotSupported {
        /// The object that exported the marker.
        path: String,
    },
    /// An MPI library outside the supported families.
    UnsupportedMpi {
        /// The library.
        path: String,
        /// Its `MPI_Get_library_version`.
        version: String,
        /// Why.
        reject: Reject,
    },
    /// A standard ABI with a major version this crate has no tables for.
    UnsupportedAbiVersion {
        /// The library.
        path: String,
        /// Its `abi_major`.
        major: i32,
    },
    /// The version string and the symbols (or the standard-ABI answer) disagree.
    FamilyMismatch {
        /// The library.
        path: String,
        /// Its version string.
        version: String,
        /// What the string says.
        string_says: String,
        /// What the symbols say.
        symbols_say: String,
    },
    /// No MPI library is loaded yet. Not cached (D12): the next call detects again.
    NoMpiInProcess,
    /// More than one MPI library is loaded.
    AmbiguousMpi {
        /// Their paths.
        paths: Vec<String>,
    },
    /// Standalone path only: nothing could be loaded by name.
    NotFound {
        /// Names tried, in order.
        tried: Vec<String>,
    },
}

const SUPPORTED: &str = "  supported: MPI 5.0 standard ABI (libmpi_abi: MPICH --enable-mpi-abi, mpi_abi_wrapper),\n             MPICH ABI (MPICH, Intel MPI, MVAPICH, Cray MPICH, HPE HMPT)\n             and Open MPI ABI (Open MPI, Fujitsu MPI, IBM Spectrum MPI)\n";

impl LoadError {
    /// `true` for the one outcome a caller may treat as "try again later".
    pub fn is_not_found(&self) -> bool {
        matches!(self, LoadError::NoMpiInProcess | LoadError::NotFound { .. })
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let issues = "https://github.com/tensor4all/tenferro-rs/issues";
        match self {
            LoadError::UnsupportedMpi { path, version, reject } => {
                let version = version.lines().next().unwrap_or("");
                write!(
                    f,
                    "tenferro-mpi: unsupported MPI library ({reject:?})\n  library: {path}\n  MPI_Get_library_version: {version:?}\n{SUPPORTED}\
tenferro passes native MPI handles and has no wrapper fallback: MPItrampoline support\n\
was removed in tensor4all/tenferro-rs#2735. Run with an MPI from a supported family\n\
(a standard-ABI build of your MPI also works), or open an issue at\n{issues} with this library's version string\n\
so its family table can be added."
                )
            }
            LoadError::MpiTrampolineNotSupported { path } => write!(
                f,
                "tenferro-mpi: the host uses MPItrampoline/MPIwrapper\n  library: {path}\n{SUPPORTED}\
tenferro passes native MPI handles and has no wrapper fallback. Select a native MPI in the host:\n\
MPIPreferences.use_system_binary(), the default MPICH_jll or MPIABI_jll for MPI.jl;\n\
a native or standard-ABI mpi4py build for Python."
            ),
            LoadError::UnsupportedAbiVersion { path, major } => write!(
                f,
                "tenferro-mpi: the MPI library implements standard ABI version {major}\n  library: {path}\n{SUPPORTED}\
tenferro knows ABI version 1; report it at {issues}."
            ),
            LoadError::FamilyMismatch { path, version, string_says, symbols_say } => {
                let version = version.lines().next().unwrap_or("");
                write!(
                    f,
                    "tenferro-mpi: version string says {string_says} but symbols say {symbols_say}\n  library: {path}\n  MPI_Get_library_version: {version:?}\n{SUPPORTED}\
this is a tenferro bug or an unknown wrapper; report it at {issues}."
                )
            }
            LoadError::NoMpiInProcess => write!(
                f,
                "tenferro-mpi: no MPI library is loaded in this process\n\
Initialize MPI in the host before attaching; a statically linked host MPI is not supported."
            ),
            LoadError::AmbiguousMpi { paths } => write!(
                f,
                "tenferro-mpi: more than one MPI library is loaded: {}\nLoad exactly one MPI.",
                paths.join(", ")
            ),
            LoadError::NotFound { tried } => write!(
                f,
                "tenferro-mpi: no MPI library could be loaded; tried {}\n\
Set MPI_NATIVE_LIB=/path/to/libmpi.so.N, or initialize MPI in the host first.",
                tried.join(", ")
            ),
        }
    }
}

impl std::error::Error for LoadError {}

/// The outcome of a successful detection: which candidate, and which family.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    /// Index into the candidate list.
    pub index: usize,
    /// The family.
    pub family: Family,
}

fn family_label(row: Row) -> String {
    match row {
        Row::Family(f) => f.name().to_string(),
        Row::Reject(r) => format!("{r:?}"),
    }
}

/// Detection order D3. The order is the contract:
///
/// 1. `MPIABI_Get_library_version` anywhere → [`LoadError::MpiTrampolineNotSupported`];
/// 2. `MPI_Abi_get_version` *returning* `abi_major >= 1` → standard ABI, decided before any
///    native witness is consulted, because an ABI library has no `ompi_mpi_comm_world`
///    symbol and (libmpi_abi does) exports `MPIR_Err_return_comm`;
/// 3. otherwise exactly one object exporting `PMPI_Get_library_version`, classified by
///    its version string;
/// 4. the per-family symbol witness in that same object.
pub fn decide(candidates: &[Candidate]) -> Result<Decision, LoadError> {
    // 1. MPIwrapper first: under MPItrampoline the trampoline, wrapper and real library
    // are all loaded, so it must be rejected before any ambiguity is counted.
    if let Some(c) = candidates.iter().find(|c| c.mpiabi_marker) {
        return Err(LoadError::MpiTrampolineNotSupported {
            path: c.path.clone(),
        });
    }

    // 2. Standard ABI: decided by the return value, never by the symbol's presence.
    let standard: Vec<usize> = candidates
        .iter()
        .enumerate()
        .filter(|(_, c)| c.abi_major.is_some_and(|m| m >= 1))
        .map(|(i, _)| i)
        .collect();
    match standard.as_slice() {
        [] => {}
        [i] => {
            let c = &candidates[*i];
            let major = c.abi_major.unwrap_or(0);
            if major > 1 {
                return Err(LoadError::UnsupportedAbiVersion {
                    path: c.path.clone(),
                    major,
                });
            }
            let version = c.version.clone().unwrap_or_default();
            return match classify_version(&version) {
                Row::Family(Family::StandardAbi) => Ok(Decision {
                    index: *i,
                    family: Family::StandardAbi,
                }),
                row => Err(LoadError::FamilyMismatch {
                    path: c.path.clone(),
                    version,
                    string_says: family_label(row),
                    symbols_say: Family::StandardAbi.name().to_string(),
                }),
            };
        }
        many => {
            return Err(LoadError::AmbiguousMpi {
                paths: many.iter().map(|i| candidates[*i].path.clone()).collect(),
            })
        }
    }

    // 3. Native ABIs: exactly one real MPI library.
    let natives: Vec<usize> = candidates
        .iter()
        .enumerate()
        .filter(|(_, c)| c.pmpi_version)
        .map(|(i, _)| i)
        .collect();
    let index = match natives.as_slice() {
        [] => return Err(LoadError::NoMpiInProcess),
        [i] => *i,
        many => {
            return Err(LoadError::AmbiguousMpi {
                paths: many.iter().map(|i| candidates[*i].path.clone()).collect(),
            })
        }
    };
    let c = &candidates[index];
    let version = c.version.clone().unwrap_or_default();
    let family = match classify_version(&version) {
        Row::Family(Family::StandardAbi) => {
            // The string claims a standard ABI but the call did not say so.
            return Err(LoadError::FamilyMismatch {
                path: c.path.clone(),
                version,
                string_says: Family::StandardAbi.name().to_string(),
                symbols_say: "native (MPI_Abi_get_version did not report abi_major >= 1)".into(),
            });
        }
        Row::Family(f) => f,
        Row::Reject(Reject::MpiWrapper) => {
            return Err(LoadError::MpiTrampolineNotSupported {
                path: c.path.clone(),
            })
        }
        Row::Reject(reject) => {
            return Err(LoadError::UnsupportedMpi {
                path: c.path.clone(),
                version,
                reject,
            })
        }
    };

    // 4. Witness, in the same object.
    let ok = match family {
        Family::OpenMpi => c.ompi_witness,
        Family::Mpich => c.mpich_witness && !c.ompi_witness,
        Family::StandardAbi => true,
    };
    if !ok {
        let symbols = if c.ompi_witness {
            Family::OpenMpi
        } else {
            Family::Mpich
        };
        return Err(LoadError::FamilyMismatch {
            path: c.path.clone(),
            version,
            string_says: family.name().to_string(),
            symbols_say: if c.ompi_witness || c.mpich_witness {
                symbols.name().to_string()
            } else {
                "no family witness symbol".to_string()
            },
        });
    }
    Ok(Decision { index, family })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MPICH: &str =
        "MPICH Version:      5.0.2\nMPICH Release date: x\nMPICH configure:    --enable-fast";
    const MPICH_ABI: &str =
        "MPICH Version:      5.0.2\nMPICH configure:    --enable-mpi-abi --prefix=/x";
    const OMPI: &str = "Open MPI v4.1.6, package: Debian OpenMPI, ident: 4.1.6, repo rev: v4.1.6";
    const INTEL: &str = "Intel(R) MPI Library 2021.19.0 for Linux* OS\n";
    const WRAPPER: &str = "mpi_abi_wrapper 1.2.0 (wrapping: Open MPI v5.0.10)";

    fn row(s: &str) -> Row {
        classify_version(s)
    }

    #[test]
    fn one_sample_per_table_row() {
        use Family::*;
        assert_eq!(
            row("MPIwrapper Version: 2.9.0"),
            Row::Reject(Reject::MpiWrapper)
        );
        assert_eq!(row(MPICH_ABI), Row::Family(StandardAbi));
        assert_eq!(row(WRAPPER), Row::Family(StandardAbi));
        assert_eq!(row(MPICH), Row::Family(Mpich));
        assert_eq!(
            row("MPICH Version:      3.0.4"),
            Row::Reject(Reject::Unknown)
        );
        assert_eq!(row(OMPI), Row::Family(OpenMpi));
        assert_eq!(row("Microsoft MPI 10.1"), Row::Reject(Reject::MsMpi));
        assert_eq!(row(INTEL), Row::Family(Mpich));
        assert_eq!(
            row("Intel(R) MPI Library 2018 Update 3"),
            Row::Family(Mpich)
        );
        assert_eq!(
            row("Intel(R) MPI Library 2014"),
            Row::Reject(Reject::Unknown)
        );
        assert_eq!(row("MVAPICH2 Version      :\t2.3.7\n"), Row::Family(Mpich));
        assert_eq!(
            row("MVAPICH Version      :\t1.2\n"),
            Row::Reject(Reject::Unknown)
        );
        assert_eq!(
            row("MPI VERSION    : CRAY MPICH version 7.7.10 (ANL base 3.2)"),
            Row::Family(Mpich)
        );
        assert_eq!(row("FUJITSU MPI 4.0"), Row::Family(OpenMpi));
        assert_eq!(row("HPE MPT 2.23  02/05/20"), Row::Reject(Reject::HpeMpt));
        assert_eq!(row("HPE HMPT 2.0"), Row::Family(Mpich));
        assert_eq!(row("NEC MPI 3.5.0 ..."), Row::Reject(Reject::Unknown));
        assert_eq!(row("garbage"), Row::Reject(Reject::Unknown));
    }

    fn cand(path: &str) -> Candidate {
        Candidate {
            path: path.into(),
            ..Default::default()
        }
    }
    fn native_ompi() -> Candidate {
        Candidate {
            pmpi_version: true,
            version: Some(OMPI.into()),
            ompi_witness: true,
            abi_major: None,
            ..cand("/usr/lib/libmpi.so.40")
        }
    }
    fn native_mpich() -> Candidate {
        Candidate {
            pmpi_version: true,
            version: Some(MPICH.into()),
            mpich_witness: true,
            abi_major: Some(-1),
            abi_minor: Some(-1),
            ..cand("/jll/libmpi.so.12")
        }
    }
    fn abi_lib() -> Candidate {
        // libmpi_abi.so.1 as observed: an MPICH --enable-mpi-abi build exports
        // MPIR_Err_return_comm and PMPI_Get_library_version too.
        Candidate {
            pmpi_version: true,
            version: Some(MPICH_ABI.into()),
            mpich_witness: true,
            abi_major: Some(1),
            abi_minor: Some(0),
            ..cand("/jll/libmpi_abi.so.1")
        }
    }

    #[test]
    fn standard_abi_whose_string_matches_no_native_row_is_not_unsupported() {
        // The audit's finding 2: an ABI library has no ompi_mpi_comm_world, and its string
        // may match no native row. The return value decides first.
        let mut c = abi_lib();
        c.version = Some(WRAPPER.into());
        c.mpich_witness = false;
        let d = decide(&[c]).unwrap();
        assert_eq!(d.family, Family::StandardAbi);
    }

    #[test]
    fn standard_abi_is_chosen_before_the_mpich_witness() {
        // Same object satisfies the MPICH witness; still the standard ABI.
        assert_eq!(decide(&[abi_lib()]).unwrap().family, Family::StandardAbi);
    }

    #[test]
    fn native_libraries_that_export_abi_get_version_return_minus_one_and_fall_through() {
        assert_eq!(decide(&[native_mpich()]).unwrap().family, Family::Mpich);
        assert_eq!(decide(&[native_ompi()]).unwrap().family, Family::OpenMpi);
    }

    #[test]
    fn wrapper_under_a_native_library_uses_the_abi_object_only() {
        // mpi_abi_wrapper sits on libmpi.so.40; the other MPI object is ignored (A9).
        let mut w = abi_lib();
        w.version = Some(WRAPPER.into());
        w.path = "/x/libmpi_abi.so.1".into();
        let d = decide(&[native_ompi(), w]).unwrap();
        assert_eq!((d.index, d.family), (1, Family::StandardAbi));
    }

    #[test]
    fn mpiwrapper_is_detected_before_ambiguity() {
        let mut marker = cand("/x/libmpiwrapper.so");
        marker.mpiabi_marker = true;
        let err = decide(&[native_ompi(), native_mpich(), marker]).unwrap_err();
        assert!(
            matches!(err, LoadError::MpiTrampolineNotSupported { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn mpiwrapper_version_string_is_rejected_by_name() {
        let mut c = native_mpich();
        c.version = Some("MPIwrapper Version: 2.9.0".into());
        assert!(matches!(
            decide(&[c]).unwrap_err(),
            LoadError::MpiTrampolineNotSupported { .. }
        ));
    }

    #[test]
    fn two_standard_abi_objects_are_ambiguous() {
        let a = abi_lib();
        let mut b = abi_lib();
        b.path = "/other/libmpi_abi.so.1".into();
        assert!(matches!(
            decide(&[a, b]).unwrap_err(),
            LoadError::AmbiguousMpi { .. }
        ));
    }

    #[test]
    fn abi_major_above_one_is_refused() {
        let mut c = abi_lib();
        c.abi_major = Some(2);
        assert!(matches!(
            decide(&[c]).unwrap_err(),
            LoadError::UnsupportedAbiVersion { major: 2, .. }
        ));
    }

    #[test]
    fn standard_abi_with_a_native_string_is_a_family_mismatch() {
        let mut c = abi_lib();
        c.version = Some(OMPI.into());
        assert!(matches!(
            decide(&[c]).unwrap_err(),
            LoadError::FamilyMismatch { .. }
        ));
    }

    #[test]
    fn native_string_claiming_standard_abi_without_the_call_is_a_mismatch() {
        let mut c = native_mpich();
        c.version = Some(MPICH_ABI.into());
        assert!(matches!(
            decide(&[c]).unwrap_err(),
            LoadError::FamilyMismatch { .. }
        ));
    }

    #[test]
    fn nothing_loaded_and_two_natives() {
        assert_eq!(decide(&[]).unwrap_err(), LoadError::NoMpiInProcess);
        assert!(matches!(
            decide(&[native_ompi(), native_mpich()]).unwrap_err(),
            LoadError::AmbiguousMpi { .. }
        ));
    }

    #[test]
    fn witness_agreement_and_disagreement() {
        let mut c = native_ompi();
        c.ompi_witness = false;
        assert!(matches!(
            decide(&[c]).unwrap_err(),
            LoadError::FamilyMismatch { .. }
        ));
        let mut c = native_mpich();
        c.mpich_witness = false;
        assert!(matches!(
            decide(&[c]).unwrap_err(),
            LoadError::FamilyMismatch { .. }
        ));
        // An MPICH string with Open MPI's data symbol is a mismatch too.
        let mut c = native_mpich();
        c.ompi_witness = true;
        assert!(matches!(
            decide(&[c]).unwrap_err(),
            LoadError::FamilyMismatch { .. }
        ));
    }

    #[test]
    fn unsupported_library_error_text_names_path_version_and_remedy() {
        let mut c = native_mpich();
        c.version = Some("NEC MPI 3.5.0 ...".into());
        c.path = "/opt/nec/ve/mpi/3.5.0/lib64/ve/libmpi.so.1".into();
        let text = decide(&[c]).unwrap_err().to_string();
        assert!(text.contains("/opt/nec/ve/mpi/3.5.0/lib64/ve/libmpi.so.1"));
        assert!(text.contains("NEC MPI 3.5.0"));
        assert!(text.contains("MPItrampoline support"));
        assert!(text.contains("tenferro-rs#2735"));
    }
}
