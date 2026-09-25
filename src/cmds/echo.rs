use crate::read::DataType;

pub fn echo(input_seq: &[DataType]) -> DataType {
    input_seq[1].clone()
}

#[cfg(test)]
mod tests {
    use crate::{cmds::echo, read::DataType};

    #[test]
    fn echo_string() {
        let val = echo::echo(&[
            DataType::NullStr,
            DataType::BulkString("My-Echo_string".to_string()),
        ]);

        assert_eq!(val, DataType::BulkString("My-Echo_string".to_string()))
    }
}
