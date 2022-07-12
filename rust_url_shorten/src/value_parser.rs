// Value that will be stored in the KV store.
pub struct Value {
    pub url: String,
    pub password: Option<String>,
}

// Error returned when deserialization of the Value fails.
pub enum DeserializeError {
    TooShort,
    TooLong,
    InternalError,
}

// Serialize the Value into bytes in a way
// that is portable across different languages.
// The result looks like this:
// | URL_LENGTH | URL DATA | PASSWORD_LENGTH | PASSWORD_DATA |
pub fn serialize_value(val: &Value) -> Vec<u8> {
    let mut serialized: Vec<u8> = Vec::new();

    // URL
    let mut url_data: Vec<u8> = val.url.clone().into_bytes();
    let mut url_size: Vec<u8> = (url_data.len() as u32).to_be_bytes().to_vec();
    serialized.append(&mut url_size);
    serialized.append(&mut url_data);

    // Password
    match &val.password {
        Some(str) => {
            let mut password_data: Vec<u8> = str.clone().into_bytes();
            let mut password_size: Vec<u8> = (password_data.len() as u32).to_be_bytes().to_vec();
            serialized.append(&mut password_size);
            serialized.append(&mut password_data);
        }
        None => {
            let mut password_size: Vec<u8> = std::u32::MAX.to_be_bytes().to_vec();
            serialized.append(&mut password_size);
        }
    }

    return serialized;
}

// Reconstruct Value from the serialized bytes, which looks like this:
// | URL_LENGTH | URL DATA | PASSWORD_LENGTH | PASSWORD_DATA |
pub fn deserialize_value(bytes: &Vec<u8>) -> Result<Value, DeserializeError> {
    let mut url: String = String::new();
    let mut password: Option<String> = None;

    enum Member {
        Url,
        Password,
        None,
    }
    enum Field {
        Size,
        Data,
    }

    let mut current_member = Member::Url;
    let mut current_field = Field::Size;

    let mut size_buf: [u8; 4] = [0; 4];
    let mut size_buf_i = 0;
    let mut size: u32 = 0;

    let mut data: Vec<u8> = Vec::new();

    for byte in bytes {
        if matches!(current_member, Member::None) {
            return Err(DeserializeError::TooLong);
        }

        match &current_field {
            Field::Size => {
                size_buf[size_buf_i] = *byte;
                size_buf_i += 1;
                if size_buf_i >= 4 {
                    size = u32::from_be_bytes(size_buf);
                    size_buf_i = 0;
                    current_field = Field::Data;
                }
            }
            Field::Data => {
                if size <= 0 {
                    return Err(DeserializeError::InternalError);
                }
                data.push(*byte);
                size -= 1;
            }
        }

        if matches!(current_field, Field::Data) {
            if size <= 0 {
                match current_member {
                    Member::Url => {
                        url = String::from_utf8(data.clone()).unwrap();
                        current_member = Member::Password;
                        current_field = Field::Size;
                        data.clear();
                    }
                    Member::Password => {
                        password = Some(String::from_utf8(data.clone()).unwrap());
                        current_member = Member::None;
                        current_field = Field::Size;
                        data.clear();
                    }
                    _ => {
                        return Err(DeserializeError::InternalError);
                    }
                }
            } else if size == u32::MAX {
                match current_member {
                    Member::Password => {
                        password = None;
                        current_member = Member::None;
                        current_field = Field::Size;
                    }
                    _ => {
                        // continue normally
                    }
                }
            }
        }
    }

    if !matches!(current_member, Member::None) {
        return Err(DeserializeError::TooShort);
    }

    return Ok(Value{url, password});
}