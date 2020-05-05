use super::rtc;
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
        assert!(has_tsc, "TSC not available");
        assert!(has_invariant_tsc, "Hardware not supported (lacks invariant tsc)");

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
        tsc_frequency_hz = cpuid.get_hypervisor_info().and_then(|hv| {
            hv.tsc_frequency().and_then(|tsc_khz| {
                Some(tsc_khz as u64 * KHZ_TO_HZ)
            })
        });

        // If we still couldn't figure it out, use RTC and measure it:
        if tsc_frequency_hz.is_none()  {
            unsafe  {
                let rtc = rtc::now();
                while rtc::now().as_unix_time() < (rtc.as_unix_time() + 1) {
                    core::arch::x86_64::_mm_pause();
                }

                let rtc = rtc::now();
                let start = x86::time::rdtsc();
                while rtc::now().as_unix_time() < (rtc.as_unix_time() + 1) {
                    core::arch::x86_64::_mm_pause();
                }
                let cycles_per_sec = x86::time::rdtsc() - start;

                tsc_frequency_hz = Some(cycles_per_sec);
            }
        }

        tsc_frequency_hz.unwrap_or_else(|| MHZ_TO_HZ*3000)
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
