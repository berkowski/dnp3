use std::time::Duration;

use crate::app::gen::count::CountVariation;
use crate::app::parse::parser::Response;
use crate::app::FunctionCode;
use crate::master::error::TaskError;
use crate::master::handle::Promise;
use crate::master::tasks::NonReadTask;

/// Type of restart to request
pub(crate) enum RestartType {
    /// Cold restart
    ///
    /// Forces the outstation to perform a complete restart similar to what the device
    /// would do upon powering up after a long-term power loss.
    ColdRestart,
    /// Warm restart
    ///
    /// Forces the outstation to perform a partial reset.
    WarmRestart,
}

pub(crate) struct RestartTask {
    restart_type: RestartType,
    promise: Promise<Result<Duration, TaskError>>,
}

impl RestartType {
    fn function(&self) -> FunctionCode {
        match self {
            Self::ColdRestart => FunctionCode::ColdRestart,
            Self::WarmRestart => FunctionCode::WarmRestart,
        }
    }
}

impl RestartTask {
    pub(crate) fn new(
        restart_type: RestartType,
        promise: Promise<Result<Duration, TaskError>>,
    ) -> Self {
        Self {
            restart_type,
            promise,
        }
    }

    pub(crate) fn wrap(self) -> NonReadTask {
        NonReadTask::Restart(self)
    }

    pub(crate) fn function(&self) -> FunctionCode {
        self.restart_type.function()
    }

    pub(crate) fn on_task_error(self, err: TaskError) {
        self.promise.complete(Err(err))
    }

    pub(crate) fn handle(self, response: Response) -> Option<NonReadTask> {
        let headers = match response.objects {
            Ok(x) => x,
            Err(err) => {
                self.promise
                    .complete(Err(TaskError::MalformedResponse(err)));
                return None;
            }
        };

        let header = match headers.get_only_header() {
            Some(x) => x,
            None => {
                self.promise
                    .complete(Err(TaskError::UnexpectedResponseHeaders));
                return None;
            }
        };

        let count = match header.details.count() {
            Some(x) => x,
            None => {
                self.promise
                    .complete(Err(TaskError::UnexpectedResponseHeaders));
                return None;
            }
        };

        match count {
            CountVariation::Group52Var1(val) => match val.single() {
                Some(val) => self
                    .promise
                    .complete(Ok(Duration::from_secs(val.time as u64))),
                None => self
                    .promise
                    .complete(Err(TaskError::UnexpectedResponseHeaders)),
            },
            CountVariation::Group52Var2(val) => match val.single() {
                Some(val) => self
                    .promise
                    .complete(Ok(Duration::from_millis(val.time as u64))),
                None => self
                    .promise
                    .complete(Err(TaskError::UnexpectedResponseHeaders)),
            },
            _ => self
                .promise
                .complete(Err(TaskError::UnexpectedResponseHeaders)),
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::app::format::write::{start_request, start_response};
    use crate::app::variations::{Group52Var1, Group52Var2};
    use crate::app::Sequence;
    use crate::app::{ControlField, Iin, ResponseFunction};
    use crate::link::EndpointAddress;
    use crate::master::association::{Association, AssociationConfig};
    use crate::master::tasks::RequestWriter;
    use crate::master::{DefaultAssociationHandler, NullReadHandler};
    use crate::util::cursor::WriteCursor;

    use super::*;

    #[test]
    fn cold_restart() {
        let mut association = Association::new(
            EndpointAddress::from(1).unwrap(),
            AssociationConfig::default(),
            NullReadHandler::boxed(),
            DefaultAssociationHandler::boxed(),
        );
        let (tx, mut rx) = crate::tokio::sync::oneshot::channel();
        let task = NonReadTask::Restart(RestartTask::new(
            RestartType::ColdRestart,
            Promise::OneShot(tx),
        ));

        // Cold restart request
        let mut buffer = [0; 20];
        let mut cursor = WriteCursor::new(&mut buffer);
        let task = task.start(&mut association).unwrap();
        let mut writer = start_request(
            ControlField::request(Sequence::default()),
            task.function(),
            &mut cursor,
        )
        .unwrap();
        task.write(&mut writer).unwrap();
        let request = writer.to_parsed().to_request().unwrap();

        assert_eq!(request.header.function, FunctionCode::ColdRestart);
        assert!(request.raw_objects.is_empty());

        // Response with delay (in seconds)
        let mut buffer = [0; 20];
        let mut cursor = WriteCursor::new(&mut buffer);
        let mut writer = start_response(
            ControlField::response(Sequence::default(), true, true, false),
            ResponseFunction::Response,
            Iin::default(),
            &mut cursor,
        )
        .unwrap();
        writer.write_count_of_one(Group52Var1 { time: 2 }).unwrap();
        let response = writer.to_parsed().to_response().unwrap();

        assert!(task.handle(&mut association, response).is_none());
        assert_eq!(rx.try_recv().unwrap(), Ok(Duration::from_secs(2)));
    }

    #[test]
    fn warm_restart() {
        let mut association = Association::new(
            EndpointAddress::from(1).unwrap(),
            AssociationConfig::default(),
            NullReadHandler::boxed(),
            DefaultAssociationHandler::boxed(),
        );
        let (tx, mut rx) = crate::tokio::sync::oneshot::channel();
        let task = NonReadTask::Restart(RestartTask::new(
            RestartType::WarmRestart,
            Promise::OneShot(tx),
        ));

        // Cold restart request
        let mut buffer = [0; 20];
        let mut cursor = WriteCursor::new(&mut buffer);
        let task = task.start(&mut association).unwrap();
        let mut writer = start_request(
            ControlField::request(Sequence::default()),
            task.function(),
            &mut cursor,
        )
        .unwrap();
        task.write(&mut writer).unwrap();
        let request = writer.to_parsed().to_request().unwrap();

        assert_eq!(request.header.function, FunctionCode::WarmRestart);
        assert!(request.raw_objects.is_empty());

        // Response with delay (in seconds)
        let mut buffer = [0; 20];
        let mut cursor = WriteCursor::new(&mut buffer);
        let mut writer = start_response(
            ControlField::response(Sequence::default(), true, true, false),
            ResponseFunction::Response,
            Iin::default(),
            &mut cursor,
        )
        .unwrap();
        writer.write_count_of_one(Group52Var2 { time: 2 }).unwrap();
        let response = writer.to_parsed().to_response().unwrap();

        assert!(task.handle(&mut association, response).is_none());
        assert_eq!(rx.try_recv().unwrap(), Ok(Duration::from_millis(2)));
    }
}
