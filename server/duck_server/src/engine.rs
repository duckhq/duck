use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use log::info;
use waithandle::{EventWaitHandle, WaitHandle};

use crate::config::ConfigurationLoader;
use crate::DuckResult;

pub struct EngineHandle {
    wait_handle: Arc<EventWaitHandle>,
    collector: JoinHandle<DuckResult<()>>,
    aggregator: JoinHandle<DuckResult<()>>,
}

impl EngineHandle {
    pub fn stop(self) -> DuckResult<()> {
        info!("Stopping engine...");
        self.wait_handle.signal()?;
        self.collector.join().unwrap()?;
        self.aggregator.join().unwrap()?;
        Ok(())
    }
}

pub struct Engine<'a> {
    _config: Box<&'a dyn ConfigurationLoader>,
}

impl<'a> Engine<'a> {
    pub fn new(config: &'a impl ConfigurationLoader) -> DuckResult<Self> {
        Ok(Engine {
            _config: Box::new(config),
        })
    }

    pub fn run(&self) -> DuckResult<EngineHandle> {
        info!("Starting engine...");
        let handle = Arc::new(EventWaitHandle::new());

        info!("Starting collector thread...");
        let collector = std::thread::spawn({
            let handle = handle.clone();
            move || -> DuckResult<()> { run_collectors(handle) }
        });

        info!("Starting aggregator thread...");
        let aggregator = std::thread::spawn({
            let handle = handle.clone();
            move || -> DuckResult<()> { run_aggregation(handle) }
        });

        Ok(EngineHandle {
            wait_handle: handle,
            collector,
            aggregator,
        })
    }
}

///////////////////////////////////////////////////////////
// Collecting

fn run_collectors(handle: Arc<dyn WaitHandle>) -> DuckResult<()> {
    loop {
        info!("Doing some work...");
        if handle.wait(Duration::from_secs(1))? {
            break;
        }
    }
    Ok(())
}

///////////////////////////////////////////////////////////
// Aggregation

fn run_aggregation(handle: Arc<dyn WaitHandle>) -> DuckResult<()> {
    loop {
        info!("Doing some aggregation work...");
        if handle.wait(Duration::from_secs(2))? {
            break;
        }
    }
    Ok(())
}
