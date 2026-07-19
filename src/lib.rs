//! Data structures to represent a GHDL elaborated design.
//!
//! AI NOTICE: Partially generated, partially reviewed.

#![allow(missing_docs, reason = "// TODO remove before release")]

mod ast_refs;
mod ids;
mod nodes;

use std::error::Error;
use std::fmt;
use std::io::BufRead;
use std::num::NonZeroU32;
use std::result::Result as StdResult;

use anyhow::Context as _;
use anyhow::Result;
use anyhow::bail;
use ghdl_ast::Ast;
use ghdl_ast::AstLoadingOutput;
use serde::Deserialize;
use tracing::debug;

pub use self::ast_refs::*;
pub use self::ids::*;
pub use self::nodes::*;

#[derive(Debug)]
pub struct Design {
    pub ast: Ast,
    pub metadata: DesignMetadata,
    pub elab_units: Vec<ElabUnit>,
    pub instances: Vec<Instance>,
    pub object_slots: Vec<ObjectSlot>,
    pub signals: Vec<Signal>,
    pub processes: Vec<Process>,
    pub drivers: Vec<Driver>,
    pub sensitivities: Vec<Sensitivity>,
    pub connections: Vec<Connection>,
    pub disconnects: Vec<Disconnect>,
    pub quantities: Vec<Quantity>,
    pub terminals: Vec<Terminal>,
    pub simultaneous: Vec<Simultaneous>,
    pub complex_simultaneous: Vec<ComplexSimultaneous>,
    pub types: Vec<TypeNode>,
    pub values: Vec<Value>,
    pub memories: Vec<Memory>,
    pub nbr_sources: Vec<NbrSources>,
}

/// Error returned when a design table ID is out of range.
#[derive(Clone, Copy, Debug)]
pub struct DesignNodeError {
    /// Design table kind that was looked up.
    pub kind: &'static str,
    /// Wire ID that was not found.
    pub id: u32,
}

impl fmt::Display for DesignNodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{kind} #{id} not found in design",
            kind = self.kind,
            id = self.id,
        )
    }
}

impl Error for DesignNodeError {}

impl Design {
    /// Returns the top-level elaborated instance, if any.
    #[must_use]
    pub fn root_instance(&self) -> Option<InstanceId> {
        self.metadata.root_instance
    }

    /// Returns the elaborated unit for `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is out of range for [`Self::elab_units`].
    pub fn elab_unit(&self, id: ElabUnitId) -> StdResult<&ElabUnit, DesignNodeError> {
        Self::get_node(&self.elab_units, id.index(), "elab_unit", id.to_raw().get())
    }

    /// Returns the instance for `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is out of range for [`Self::instances`].
    pub fn instance(&self, id: InstanceId) -> StdResult<&Instance, DesignNodeError> {
        Self::get_node(&self.instances, id.index(), "instance", id.to_raw().get())
    }

    /// Returns the signal for `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is out of range for [`Self::signals`].
    pub fn signal(&self, id: SignalId) -> StdResult<&Signal, DesignNodeError> {
        Self::get_node(&self.signals, id.index(), "signal", id.to_raw().get())
    }

    /// Returns the process for `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is out of range for [`Self::processes`].
    pub fn process(&self, id: ProcessId) -> StdResult<&Process, DesignNodeError> {
        Self::get_node(&self.processes, id.index(), "process", id.to_raw().get())
    }

    /// Returns the driver for `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is out of range for [`Self::drivers`].
    pub fn driver(&self, id: DriverId) -> StdResult<&Driver, DesignNodeError> {
        Self::get_node(&self.drivers, id.index(), "driver", id.to_raw().get())
    }

    /// Returns the sensitivity for `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is out of range for [`Self::sensitivities`].
    pub fn sensitivity(&self, id: SensitivityId) -> StdResult<&Sensitivity, DesignNodeError> {
        Self::get_node(
            &self.sensitivities,
            id.index(),
            "sensitivity",
            id.to_raw().get(),
        )
    }

    /// Returns the connection for `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is out of range for [`Self::connections`].
    pub fn connection(&self, id: ConnectionId) -> StdResult<&Connection, DesignNodeError> {
        Self::get_node(
            &self.connections,
            id.index(),
            "connection",
            id.to_raw().get(),
        )
    }

    /// Returns the disconnect for `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is out of range for [`Self::disconnects`].
    pub fn disconnect(&self, id: DisconnectId) -> StdResult<&Disconnect, DesignNodeError> {
        Self::get_node(
            &self.disconnects,
            id.index(),
            "disconnect",
            id.to_raw().get(),
        )
    }

    /// Returns the quantity for `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is out of range for [`Self::quantities`].
    pub fn quantity(&self, id: QuantityId) -> StdResult<&Quantity, DesignNodeError> {
        Self::get_node(&self.quantities, id.index(), "quantity", id.to_raw().get())
    }

    /// Returns the terminal for `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is out of range for [`Self::terminals`].
    pub fn terminal(&self, id: TerminalId) -> StdResult<&Terminal, DesignNodeError> {
        Self::get_node(&self.terminals, id.index(), "terminal", id.to_raw().get())
    }

    /// Returns the simultaneous statement for `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is out of range for [`Self::simultaneous`].
    pub fn simultaneous(&self, id: SimultaneousId) -> StdResult<&Simultaneous, DesignNodeError> {
        Self::get_node(
            &self.simultaneous,
            id.index(),
            "simultaneous",
            id.to_raw().get(),
        )
    }

    /// Returns the complex simultaneous statement for `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is out of range for [`Self::complex_simultaneous`].
    pub fn complex_simultaneous(
        &self,
        id: ComplexSimultaneousId,
    ) -> StdResult<&ComplexSimultaneous, DesignNodeError> {
        Self::get_node(
            &self.complex_simultaneous,
            id.index(),
            "complex_simultaneous",
            id.to_raw().get(),
        )
    }

    /// Returns the type node for `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is out of range for [`Self::types`].
    pub fn typ(&self, id: TypeId) -> StdResult<&TypeNode, DesignNodeError> {
        Self::get_node(&self.types, id.index(), "type", id.to_raw().get())
    }

    /// Returns the value for `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is out of range for [`Self::values`].
    pub fn value(&self, id: ValueId) -> StdResult<&Value, DesignNodeError> {
        Self::get_node(&self.values, id.index(), "value", id.to_raw().get())
    }

    /// Returns the memory for `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is out of range for [`Self::memories`].
    pub fn memory(&self, id: MemoryId) -> StdResult<&Memory, DesignNodeError> {
        Self::get_node(&self.memories, id.index(), "memory", id.to_raw().get())
    }

    /// Returns the number-of-sources entry for `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `id` is out of range for [`Self::nbr_sources`].
    pub fn nbr_sources(&self, id: NbrSourcesId) -> StdResult<&NbrSources, DesignNodeError> {
        Self::get_node(
            &self.nbr_sources,
            id.index(),
            "nbr_sources",
            id.to_raw().get(),
        )
    }

    /// Looks up `index` in `table`, mapping out-of-range IDs to [`DesignNodeError`].
    fn get_node<'a, T>(
        table: &'a [T],
        index: usize,
        kind: &'static str,
        id: u32,
    ) -> StdResult<&'a T, DesignNodeError> {
        table.get(index).ok_or(DesignNodeError { kind, id })
    }

    /// Constructs a `Design` from a JSON stream.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the buffer or parsing the JSON fails,
    /// or if a design node ID is not contiguous (except skipped instance IDs).
    pub fn from_json(reader: &mut dyn BufRead) -> Result<Design> {
        let AstLoadingOutput {
            ast,
            mut next_line_number,
        } = Ast::from_json(reader, 1)?;

        let mut line_buffer = String::new();
        reader.read_line(&mut line_buffer)?;
        let metadata: DesignMetadata =
            serde_json::from_str(&line_buffer).context("could not parse design metadata")?;
        debug!("Design metadata: {metadata:#?}");

        let DesignCounts {
            elab_unit,
            instance,
            signal,
            process,
            driver,
            sensitivity,
            connection,
            disconnect,
            quantity,
            terminal,
            simultaneous,
            complex_simultaneous,
            typ,
            value,
            memory,
            nbr_sources,
        } = metadata.counts;

        let mut design = Design {
            ast,
            metadata,
            elab_units: Vec::with_capacity(elab_unit as usize),
            instances: Vec::with_capacity(instance as usize),
            object_slots: Vec::new(),
            signals: Vec::with_capacity(signal as usize),
            processes: Vec::with_capacity(process as usize),
            drivers: Vec::with_capacity(driver as usize),
            sensitivities: Vec::with_capacity(sensitivity as usize),
            connections: Vec::with_capacity(connection as usize),
            disconnects: Vec::with_capacity(disconnect as usize),
            quantities: Vec::with_capacity(quantity as usize),
            terminals: Vec::with_capacity(terminal as usize),
            simultaneous: Vec::with_capacity(simultaneous as usize),
            complex_simultaneous: Vec::with_capacity(complex_simultaneous as usize),
            types: Vec::with_capacity(typ as usize),
            values: Vec::with_capacity(value as usize),
            memories: Vec::with_capacity(memory as usize),
            nbr_sources: Vec::with_capacity(nbr_sources as usize),
        };

        loop {
            next_line_number += 1;
            line_buffer.clear();
            reader.read_line(&mut line_buffer)?;
            let line = line_buffer.trim();
            if line.is_empty() {
                break;
            }

            let node = serde_json::from_str::<Node>(line)
                .with_context(|| format!("parse error in line {next_line_number}: {line}"))?;
            design
                .add_node(node)
                .with_context(|| format!("invalid design node in line {next_line_number}"))?;
        }

        Ok(design)
    }

    /// Appends a design node, enforcing contiguous table IDs.
    ///
    /// Instance IDs may skip values; missing slots are filled with
    /// [`Instance::dummy`]. All other ID'd kinds must arrive in order with no gaps.
    ///
    /// # Errors
    ///
    /// Returns an error if a non-instance ID is not the next contiguous index,
    /// or if an instance ID is out of order (already past that index).
    fn add_node(&mut self, node: Node) -> Result<()> {
        match node {
            Node::ElabUnit(node) => push_contiguous(&mut self.elab_units, node, "elab_unit"),
            Node::Instance(node) => self.add_instance(node),
            Node::ObjectSlot(node) => {
                self.object_slots.push(node);
                Ok(())
            },
            Node::Signal(node) => push_contiguous(&mut self.signals, node, "signal"),
            Node::Process(node) => push_contiguous(&mut self.processes, node, "process"),
            Node::Driver(node) => push_contiguous(&mut self.drivers, node, "driver"),
            Node::Sensitivity(node) => {
                push_contiguous(&mut self.sensitivities, node, "sensitivity")
            },
            Node::Connection(node) => push_contiguous(&mut self.connections, node, "connection"),
            Node::Disconnect(node) => push_contiguous(&mut self.disconnects, node, "disconnect"),
            Node::Quantity(node) => push_contiguous(&mut self.quantities, node, "quantity"),
            Node::Terminal(node) => push_contiguous(&mut self.terminals, node, "terminal"),
            Node::Simultaneous(node) => {
                push_contiguous(&mut self.simultaneous, node, "simultaneous")
            },
            Node::ComplexSimultaneous(node) => {
                push_contiguous(&mut self.complex_simultaneous, node, "complex_simultaneous")
            },
            Node::Type(node) => push_contiguous(&mut self.types, node, "type"),
            Node::Value(node) => push_contiguous(&mut self.values, node, "value"),
            Node::Memory(node) => push_contiguous(&mut self.memories, node, "memory"),
            Node::NbrSources(node) => push_contiguous(&mut self.nbr_sources, node, "nbr_sources"),
        }
    }

    /// Appends an instance, inserting [`Instance::dummy`] entries for skipped IDs.
    fn add_instance(&mut self, node: Instance) -> Result<()> {
        let index = node.id.index();
        if index < self.instances.len() {
            bail!(
                "instance ID {id} is out of order (expected index >= {len})",
                id = node.id,
                len = self.instances.len(),
            );
        }
        while self.instances.len() < index {
            let raw = u32::try_from(self.instances.len() + 1).context("instance table overflow")?;
            let id = NonZeroU32::new(raw)
                .map(InstanceId::from_raw)
                .context("instance ID cannot be zero")?;
            self.instances.push(Instance::dummy(id));
        }
        self.instances.push(node);
        Ok(())
    }
}

/// Pushes `node` when `index` is the next contiguous slot in `table`.
fn push_contiguous<T: IdIndex>(table: &mut Vec<T>, node: T, kind: &str) -> Result<()> {
    let index = node.id_index();
    let expected = table.len();
    if index != expected {
        bail!("{kind} ID index {index} is not contiguous (expected {expected})");
    }
    table.push(node);
    Ok(())
}

#[derive(Clone, Debug, Deserialize)]
pub struct DesignMetadata {
    pub schema: u32,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub root_instance: Option<InstanceId>,
    pub counts: DesignCounts,
    pub name: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DesignCounts {
    pub elab_unit: u32,
    pub instance: u32,
    pub signal: u32,
    pub process: u32,
    pub driver: u32,
    pub sensitivity: u32,
    pub connection: u32,
    pub disconnect: u32,
    pub quantity: u32,
    pub terminal: u32,
    pub simultaneous: u32,
    pub complex_simultaneous: u32,
    #[serde(rename = "type")]
    pub typ: u32,
    pub value: u32,
    pub memory: u32,
    pub nbr_sources: u32,
}
