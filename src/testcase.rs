use std::fmt::Display;

pub trait TCAssert1<T, A> {
    fn assert(self, msg: A) -> T;
}

pub trait TCAssert0<T> {
    fn assert(self) -> T;
}

impl<T, E> TCAssert1<T, &str> for Result<T, E>
where
    E: Display,
{
    fn assert(self, msg: &str) -> T {
        self.unwrap_or_else(|e| panic!("Assertion failed: {}\nError: {}", msg, e))
    }
}

impl<T, E> TCAssert0<T> for Result<T, E>
where
    E: Display,
{
    fn assert(self) -> T {
        self.unwrap_or_else(|e| panic!("Assertion failed \nError: {}", e))
    }
}

impl<T, E> TCAssert1<T, (&str, &str)> for Result<T, E>
where
    E: Display,
{
    fn assert(self, args: (&str, &str)) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                //assert!(false,"{}\n{}",msg,e);
                unreachable!("result assert fail! {} {} \n{}", args.0, args.1, e);
                //panic!()
                //panic!("called `Result::unwrap()` on an `Err` value", &e),
            }
        }
    }
}

impl<T, E> TCAssert1<T, (&str, &str, &str)> for Result<T, E>
where
    E: Display,
{
    fn assert(self, args: (&str, &str, &str)) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                //assert!(false,"{}\n{}",msg,e);
                unreachable!(
                    "result assert fail! {} {} {} \n{}",
                    args.0, args.1, args.2, e
                );
                //panic!()
                //panic!("called `Result::unwrap()` on an `Err` value", &e),
            }
        }
    }
}

impl<T> TCAssert0<T> for Option<T> {
    fn assert(self) -> T {
        match self {
            Some(t) => t,
            None => {
                unreachable!("is none!");
            }
        }
    }
}

impl<T> TCAssert1<T, &str> for Option<T> {
    fn assert(self, args: &str) -> T {
        match self {
            Some(t) => t,
            None => {
                unreachable!("is none! {}", args);
            }
        }
    }
}

impl<T> TCAssert1<T, (&str, &str)> for Option<T> {
    fn assert(self, args: (&str, &str)) -> T {
        match self {
            Some(t) => t,
            None => {
                unreachable!("is none! {} {}", args.0, args.1);
            }
        }
    }
}
