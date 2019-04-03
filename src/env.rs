// 1. background thread
// 2. file name allocation
#[derive(Debug, Copy, Clone)]
pub struct Env{
    file_number_: u64,
}

impl Env{
    pub fn new() -> Env{
        Env{
            file_number_: 0,
        }
    }
    pub fn get_file_name(&mut self) -> String{
        self.file_number_ += 1;
        format!("{}.df",self.file_number_)
    }
}