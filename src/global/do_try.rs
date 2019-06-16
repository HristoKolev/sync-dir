use super::prelude::*;

pub struct FinStruct<TRes> {
    result: Result<TRes>
}

impl<TRes> FinStruct<TRes> {

    pub fn finally<FFinally>(self, ff: FFinally) -> Result<TRes>
        where FFinally : FnOnce() -> Result {

        let finally_result = ff();

        match self.result {
            Err(err) => Err(err),
            Ok(res) => {
                if let Err(finally_err) = finally_result {
                    Err(finally_err)
                } else {
                    Ok(res)
                }
            }
        }
    }
}

pub fn run<FDo, TRes>(fdo: FDo) -> FinStruct<TRes>
    where FDo: FnOnce() -> Result<TRes> {

    let result = fdo();

    FinStruct {
        result
    }
}
