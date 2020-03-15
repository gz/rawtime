use crate::DateTime;

pub mod tsc {
    use crate::ONE_GHZ_IN_HZ;

    lazy_static! {
        /// TSC Frequency in Hz
        pub static ref TSC_FREQUENCY: u64 = {
            let cpuid = x86::cpuid::CpuId::new();
            let has_tsc = cpuid
                .get_feature_info()
                .map_or(false, |finfo| finfo.has_tsc());
            assert!(has_tsc);

            let tsc_frequency = cpuid
                .get_tsc_info()
                .map_or(0, |tinfo| tinfo.tsc_frequency());

            if tsc_frequency != 0 {
                tsc_frequency
            }
            else {
                cpuid
                    .get_processor_frequency_info()
                    .map_or(3*ONE_GHZ_IN_HZ, |pinfo| pinfo.processor_max_frequency() as u64 * 1000000) as u64
            }

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
