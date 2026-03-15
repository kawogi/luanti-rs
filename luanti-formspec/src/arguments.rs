use std::{collections::VecDeque, num::ParseIntError};

use crate::error::ArgumentError;

#[derive(Debug)]
pub(crate) struct RawArgs<'formspec> {
    pub(crate) args: VecDeque<Vec<&'formspec str>>,
}

impl RawArgs<'_> {
    pub(crate) fn read_u32(&mut self) -> Result<Option<u32>, ArgumentError> {
        let Some(sub_args) = self.args.pop_front() else {
            return Ok(None);
        };
        match *sub_args.into_boxed_slice() {
            [] => Ok(None),
            [arg_str] => arg_str
                .parse()
                .map_err(ArgumentError::ParseIntError)
                .map(Some),
            ref excessive => Err(ArgumentError::Excessive(format!("{excessive:?}"))),
        }
    }

    pub(crate) fn check_empty(self) -> Result<(), String> {
        if self.args.is_empty() {
            Ok(())
        } else {
            Err(format!("{:?}", self.args))
        }
    }
}
