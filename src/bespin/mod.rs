use crate::DateTime;

pub mod tsc {
    use crate::ONE_GHZ_IN_HZ;

    lazy_static! {
        /// TSC Frequency in Hz
        pub static ref TSC_FREQUENCY: u64 = {
            const MHZ_TO_HZ: u64 = 1000000;
            const KHZ_TO_HZ: u64 = 1000;

            let cpuid = x86::cpuid::CpuId::new();
            let has_tsc = cpuid
                .get_feature_info()
                .map_or(false, |finfo| finfo.has_tsc());

            let has_invariant_tsc = cpuid
                .get_extended_function_info()
                .map_or(false, |efinfo| efinfo.has_invariant_tsc());
            assert!(has_invariant_tsc, "Hardware not supported (lacks invariant tsc)");

            let tsc_frequency_hz = cpuid.get_tsc_info().map(|tinfo| {
                if tinfo.nominal_frequency() != 0 {
                    Some(tinfo.tsc_frequency())
                } else if tinfo.numerator() != 0 && tinfo.denominator() != 0 {
                    // Skylake and Kabylake don't report the crystal clock, approximate with base frequency:
                    cpuid
                        .get_processor_frequency_info()
                        .map(|pinfo| pinfo.processor_base_frequency() as u64 * MHZ_TO_HZ)
                        .map(|cpu_base_freq_hz| {
                            let crystal_hz =
                                cpu_base_freq_hz * tinfo.denominator() as u64 / tinfo.numerator() as u64;
                            crystal_hz * tinfo.numerator() as u64 / tinfo.denominator() as u64
                        })
                } else {
                    None
                }
            });

            tsc_frequency_hz.unwrap_or_else(|| {
                // Maybe we run in a VM and the hypervisor can give us the TSC frequency
                cpuid.get_hypervisor_info().map(|hv| {
                    hv.tsc_frequency().map(|tsc_khz| {
                        tsc_khz as u64 * KHZ_TO_HZ
                    })
                })
                .unwrap_or_else(|| panic!("Couldn't determine the TSC frequency"))
            }).unwrap_or_else(|| panic!("Couldn't determine the TSC frequency"))
        };
    }

    #[inline]
    fn tsc_to_ns(hz: u64) -> u64 {
        (hz as f64 / (*TSC_FREQUENCY as f64 / ONE_GHZ_IN_HZ as f64)) as u64
    }

    #[inline]
    pub fn precise_time_ns() -> u64 {
        unsafe { tsc_to_ns(x86::time::rdtsc()) as u64 }
    }
}

pub fn wallclock() -> DateTime {
    DateTime {
        sec: 1 as u8,
        min: 1 as u8,
        day: 1 as u8,
        hour: 1 as u8,
        mon: 1 as u8,
        year: 1900 as u64,
    }
}

pub fn precise_time_ns() -> u64 {
    tsc::precise_time_ns()
}
