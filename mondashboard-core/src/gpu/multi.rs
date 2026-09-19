use super::{AllGpuStats, amd, amd_igpu, intel, nvidia};

pub(super) fn collect_all() -> AllGpuStats {
    let mut gpus = vec![];
    gpus.extend(nvidia::detect());
    gpus.extend(amd::detect());
    gpus.extend(amd_igpu::detect());
    gpus.extend(intel::detect());
    AllGpuStats { gpus }
}
