use detail_error::DetailError;

#[derive(DetailError)]
pub enum BusinessError {
    InvalidEmail,
    #[detail(code=500, message="this is an invalid password")]
    InvalidPassword
}

fn main() {
    let error = BusinessError::InvalidPassword;
    println!("error {} {} {}",error.get_http_code(), error.get_code(),error.get_message());
    let error = BusinessError::InvalidEmail;
    println!("error {} {} {}",error.get_http_code(), error.get_code(),error.get_message());
}
