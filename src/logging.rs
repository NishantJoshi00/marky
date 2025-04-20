macro_rules! info {
        ($self:expr, $($arg:tt)*) => {
            $self.client.log_message(lsp_types::MessageType::INFO, format!($($arg)*)).await
        };
    }

macro_rules! error {
        ($self:expr, $($arg:tt)*) => {
            $self.client.log_message(lsp_types::MessageType::ERROR, format!($($arg)*)).await
        };
    }

macro_rules! warn {
        ($self:expr, $($arg:tt)*) => {
            $self.client.log_message(lsp_types::MessageType::WARNING, format!($($arg)*)).await
        };
    }

#[allow(unused_macros)]
macro_rules! debug {
        ($self:expr, $($arg:tt)*) => {
            $self.client.log_message(lsp_types::MessageType::LOG, format!($($arg)*)).await
        };
    }
