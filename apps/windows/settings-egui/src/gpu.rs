//! spike 的 wgpu 适配器选择：缺省交给 eframe（会挑独显 / 核显），
//! 设了 `QINGJIAN_SPIKE_SOFTWARE=1` 就强挑 CPU 适配器（D3D12 的 WARP），
//! 用来模拟没有可用 GPU 的机器与 RDP 会话——那是 glow（OpenGL 3.3）起不来、wgpu 还能活的场景。

use std::sync::Arc;

use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup};

/// 置上就强走软件适配器。
const SOFTWARE_ENV: &str = "QINGJIAN_SPIKE_SOFTWARE";

pub(crate) fn configuration() -> WgpuConfiguration {
    let mut config = WgpuConfiguration::default();
    if std::env::var_os(SOFTWARE_ENV).is_none() {
        return config;
    }
    if let WgpuSetup::CreateNew(setup) = &mut config.wgpu_setup {
        setup.native_adapter_selector = Some(Arc::new(|adapters, _surface| {
            adapters
                .iter()
                .find(|adapter| adapter.get_info().device_type == wgpu::DeviceType::Cpu)
                .cloned()
                .ok_or_else(|| "这台机器上没有 CPU（WARP）适配器".to_owned())
        }));
    }
    config
}
