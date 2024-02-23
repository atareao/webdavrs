


pub struct Response<'a, T>{
    pub status: bool,
    pub message: &'a str,
    pub data: Option<T>,
}
