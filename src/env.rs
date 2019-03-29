// 1. background thread
// 2. file name allocation

struct Env{
    file_number_: u64,
}

impl Env{
    pub fn GetFileName() -> String{
        format!("{}.df",file_number_)
    }
}