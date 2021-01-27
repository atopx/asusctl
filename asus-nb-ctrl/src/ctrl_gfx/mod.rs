pub mod vendors;

pub mod error;

pub mod gfx;

pub mod system;

const PRIME_DISCRETE_PATH: &str = "/etc/prime-discrete";
const MODPROBE_PATH: &str = "/etc/modprobe.d/asusd.conf";
const INITRAMFS_PATH: &str = "/usr/sbin/update-initramfs";
const DRACUT_PATH: &str = "/usr/bin/dracut";

static MODPROBE_NVIDIA: &[u8] = MODPROBE_HYBRID;

static MODPROBE_HYBRID: &[u8] = br#"# Automatically generated by asusd
blacklist i2c_nvidia_gpu
alias i2c_nvidia_gpu off
options nvidia NVreg_DynamicPowerManagement=0x02
options nvidia-drm modeset=1
"#;

static MODPROBE_COMPUTE: &[u8] = br#"# Automatically generated by asusd
blacklist i2c_nvidia_gpu
alias i2c_nvidia_gpu off
options nvidia NVreg_DynamicPowerManagement=0x02
options nvidia-drm modeset=0
"#;

static MODPROBE_INTEGRATED: &[u8] = br#"# Automatically generated by asusd
blacklist i2c_nvidia_gpu
blacklist nouveau
blacklist nvidia
blacklist nvidia-drm
blacklist nvidia-modeset
alias i2c_nvidia_gpu off
alias nouveau off
alias nvidia off
alias nvidia-drm off
alias nvidia-modeset off
"#;

const PRIMARY_GPU_XORG_PATH: &str = "/etc/X11/xorg.conf.d/90-nvidia-primary.conf";

static PRIMARY_GPU_BEGIN: &[u8] = br#"# Automatically generated by asusd
Section "OutputClass"
    Identifier "nvidia"
    MatchDriver "nvidia-drm"
    Driver "nvidia"
    Option "AllowEmptyInitialConfiguration"
    Option "AllowExternalGpus""#;

static PRIMARY_GPU_NVIDIA: &[u8] = br#"
    Option "PrimaryGPU" "true""#;

static PRIMARY_GPU_END: &[u8] = br#"
EndSection"#;
