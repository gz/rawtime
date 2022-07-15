use crate::DateTime;

fn determine_cpu_frequency() -> u64 {
    const MHZ_TO_HZ: u64 = 1000000;
    const KHZ_TO_HZ: u64 = 1000;
    let cpuid = x86::cpuid::CpuId::new();

    // Use info from hypervisor if available:
    if let Some(hv) = cpuid.get_hypervisor_info() {
        if let Some(tsc_khz) = hv.tsc_frequency() {
            return tsc_khz as u64 * KHZ_TO_HZ;
        }
    }

    // Use CpuId info if available:
    if let Some(tinfo) = cpuid.get_tsc_info() {
        if let Some(freq) = tinfo.tsc_frequency() {
            return freq;
        } else {
            if tinfo.numerator() != 0 && tinfo.denominator() != 0 {
                // Approximate with the processor frequency:
                if let Some(pinfo) = cpuid.get_processor_frequency_info() {
                    let cpu_base_freq_hz = pinfo.processor_base_frequency() as u64 * MHZ_TO_HZ;
                    let crystal_hz =
                        cpu_base_freq_hz * tinfo.denominator() as u64 / tinfo.numerator() as u64;
                    return crystal_hz * tinfo.numerator() as u64 / tinfo.denominator() as u64;
                }
            }
        }
    }

    3000 * MHZ_TO_HZ
}

pub mod tsc {
    use crate::ONE_GHZ_IN_HZ;

    lazy_static! {
        /// TSC Frequency in Hz
        pub static ref TSC_FREQUENCY: u64 = {
            super::determine_cpu_frequency()
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
