pub mod model_management;
pub mod py;
pub mod tokenizer;

pub enum Device {
    CPU,
    CUDA,
}

impl ToString for Device {
    fn to_string(&self) -> String {
        match self {
            Device::CPU => "cpu".to_string(),
            Device::CUDA => "cuda".to_string(),
        }
    }
}

impl Device {
    #[allow(dead_code)]
    fn is_cuda(&self) -> bool {
        match self {
            Device::CPU => false,
            Device::CUDA => true,
        }
    }

    #[allow(dead_code)]
    fn auto() -> Self {
        //TODO:
        Device::CPU
    }
}
