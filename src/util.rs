pub fn die(message: &str) -> ! {
    eprintln!("{}", message);
    std::process::exit(1);
}

pub trait OrDie {
    type Out;
    fn or_die(self, message: &str) -> Self::Out;
}

impl<T> OrDie for Option<T> {
    type Out = T;

    fn or_die(self, message: &str) -> Self::Out {
        match self {
            Some(x) => x,
            None => die(message),
        }
    }
}

impl<T, E> OrDie for Result<T, E> {
    type Out = T;

    fn or_die(self, message: &str) -> Self::Out {
        match self {
            Ok(x) => x,
            Err(_) => die(message),
        }
    }
}

/// Parses a value as a number
pub fn parse_number(value: &str) -> Option<i32> {
    if let Ok(x) = value.parse::<i32>() {
        return Some(x);
    }
    if value.len() == 9 {
        if &value[8..9] == "b" {
            if let Ok(x) = i32::from_str_radix(&value[0..8], 2) {
                return Some(x);
            }
        }
    }
    if value.len() == 17 {
        if &value[16..17] == "b" {
            if let Ok(x) = i32::from_str_radix(&value[0..16], 2) {
                return Some(x);
            }
        }
    }
    if value.len() > 2 {
        if &value[0..2] == "0x" {
            if let Ok(x) = i32::from_str_radix(&value[2..], 16) {
                return Some(x);
            }
        }
    }
    None
}
