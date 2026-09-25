use crate::read::DataType;

pub fn ping() -> DataType {
    DataType::SimpleStr("PONG".to_string())
}

#[cfg(test)]
mod tests {
    use crate::{cmds::ping, read::DataType};

    #[test]
    fn pong() {
        let val = ping::ping();

        assert_eq!(val, DataType::SimpleStr("PONG".to_string()))
    }
}
