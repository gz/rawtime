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
            assert!(has_tsc, "TSC not available");

            // Determine frequency from CPUID:
            let mut tsc_frequency_hz = cpuid.get_tsc_info().and_then(|tinfo| {
                match tinfo.tsc_frequency() {
                    None => {
                        if tinfo.numerator() != 0 && tinfo.denominator() != 0 {
                            cpuid
                            .get_processor_frequency_info()
                            .and_then(|pinfo| Some(pinfo.processor_base_frequency() as u64 * MHZ_TO_HZ))
                            .and_then(|cpu_base_freq_hz| {
                                let crystal_hz =
                                    cpu_base_freq_hz * tinfo.denominator() as u64 / tinfo.numerator() as u64;
                                Some(crystal_hz * tinfo.numerator() as u64 / tinfo.denominator() as u64)
                            })
                        }
                        else {
                            None
                        }
                    }
                    frequency => frequency,
                }
            });

            // Override with info from hypervisor if available:
            let tsc_frequency_hz_vm = cpuid.get_hypervisor_info().and_then(|hv| {
                hv.tsc_frequency()
                    .and_then(|tsc_khz| Some(tsc_khz as u64 * KHZ_TO_HZ))
            });
            if tsc_frequency_hz_vm.is_some() {
                tsc_frequency_hz = tsc_frequency_hz_vm;
            }

            tsc_frequency_hz.expect("Couldn't determine TSC frequency")
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
