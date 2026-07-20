//! Design node types matching GHDL's `design_export.adb` JSON output.
//!
//! These types are the Rust view of an elaborated VHDL design hierarchy
//! (IEEE 1076 §14): instances with object slots, processes interconnected by
//! signals/nets, and interned types/values/memories used at runtime.
//!
//! Source Ada packages (GHDL):
//! - [`Elab.Vhdl_Context`](ghdl/src/synth/elab-vhdl_context.ads) — instances, object slots
//! - [`Elab.Vhdl_Objtypes`](ghdl/src/synth/elab-vhdl_objtypes.ads) — elaborated types
//! - [`Elab.Vhdl_Values`](ghdl/src/synth/elab-vhdl_values.ads) — value decomposition
//! - [`Simul.Vhdl_Elab`](ghdl/src/simul/simul-vhdl_elab.ads) — signals, processes, drivers, connections
//! - [`Design_Export`](ghdl/src/design_export.adb) — JSON emission
//!
//! AI NOTICE: Mostly generated, minimally reviewed.

use std::fmt;

use ghdl_ast::ConstantDeclaration;
use ghdl_ast::Direction;
use ghdl_ast::NodeId;
use ghdl_ast::TypeAndSubtypeDefinitionNodeId;
use ghdl_ast::deserialize_f64;
use ghdl_ast::deserialize_optional_node_id;
use serde::Deserialize;
use serde::Deserializer;
use serde::de::Error as _;

use crate::ConnectionAssociationNodeId;
use crate::ConnectionId;
use crate::Design;
use crate::DesignSlotDeclarationNodeId;
use crate::DisconnectId;
use crate::DriverId;
use crate::InstanceBlockRefNodeId;
use crate::InstanceConfigurationNodeId;
use crate::InstanceId;
use crate::InstanceSourceNodeId;
use crate::InstanceStatementNodeId;
use crate::MemoryId;
use crate::NbrSourcesId;
use crate::ProcessId;
use crate::ProcessStmtNodeId;
use crate::RecordFieldDeclNodeId;
use crate::SensitivityId;
use crate::SignalDeclNodeId;
use crate::SignalId;
use crate::TypeId;
use crate::UninstantiatedScopeRefNodeId;
use crate::ValueId;
use crate::deserialize_optional_id;

/// Top-level tagged design node as emitted on one JSONL line.
///
/// Each variant corresponds to a GHDL table entry or interned object.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Node {
    Instance(Instance),
    ObjectSlot(ObjectSlot),
    Signal(Signal),
    Process(Process),
    Driver(Driver),
    Sensitivity(Sensitivity),
    Connection(Connection),
    Disconnect(Disconnect),
    #[serde(rename = "type")]
    Type(TypeNode),
    Value(Value),
    Memory(Memory),
    NbrSources(NbrSources),
}

impl Node {
    /// Returns the JSON tag string for this node kind.
    #[must_use]
    pub fn type_str(&self) -> &'static str {
        match self {
            Self::Instance(..) => "instance",
            Self::ObjectSlot(..) => "object_slot",
            Self::Signal(..) => "signal",
            Self::Process(..) => "process",
            Self::Driver(..) => "driver",
            Self::Sensitivity(..) => "sensitivity",
            Self::Connection(..) => "connection",
            Self::Disconnect(..) => "disconnect",
            Self::Type(..) => "type",
            Self::Value(..) => "value",
            Self::Memory(..) => "memory",
            Self::NbrSources(..) => "nbr_sources",
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Instance(inner) => fmt::Debug::fmt(inner, formatter),
            Self::ObjectSlot(inner) => fmt::Debug::fmt(inner, formatter),
            Self::Signal(inner) => fmt::Debug::fmt(inner, formatter),
            Self::Process(inner) => fmt::Debug::fmt(inner, formatter),
            Self::Driver(inner) => fmt::Debug::fmt(inner, formatter),
            Self::Sensitivity(inner) => fmt::Debug::fmt(inner, formatter),
            Self::Connection(inner) => fmt::Debug::fmt(inner, formatter),
            Self::Disconnect(inner) => fmt::Debug::fmt(inner, formatter),
            Self::Type(inner) => fmt::Debug::fmt(inner, formatter),
            Self::Value(inner) => fmt::Debug::fmt(inner, formatter),
            Self::Memory(inner) => fmt::Debug::fmt(inner, formatter),
            Self::NbrSources(inner) => fmt::Debug::fmt(inner, formatter),
        }
    }
}

/// Indexes a packed design table via a 1-based wire ID.
pub trait IdIndex {
    /// Returns the 0-based index into the corresponding packed table (`raw - 1`).
    #[must_use]
    fn id_index(&self) -> usize;
}

/// Elaborated scope in the design hierarchy (`Synth_Instance_Type`).
///
/// Parallel to a simulation block instance: each architecture, block, generate
/// iteration, process, subprogram activation, package instance, or foreign
/// module gets one entry. Occupied object slots are exported separately as
/// [`ObjectSlot`] entries.
///
/// # `stmt` vs `source` vs `block`
///
/// | Field | Meaning | Typical IIR |
/// |-------|---------|-------------|
/// | [`stmt`](Self::stmt) | Construct that *created* this scope | instantiation, process, generate, call |
/// | [`source`](Self::source) | Construct that *defines* the scope | architecture body, component, package |
/// | [`block`](Self::block) | Scope identity used for object lookup | entity for architectures (`Get_Info_Scope`) |
///
/// For a top-level architecture instance, `source` is the architecture body
/// while `block` is usually the **entity** (GHDL maps architecture bodies to
/// the entity annotation for lookup). Root, package, and verification-unit instances have
/// `stmt == None`.
///
/// ```vhdl
/// entity e is end;
/// architecture a of e is
/// begin
///   b: block begin end block;           -- instance: stmt=b, source=b
///   g: for i in 0 to 3 generate
///   begin end generate;                 -- one instance per iteration
///   u: entity work.child port map (...); -- stmt=u, source=child's architecture
/// end;
/// ```
#[derive(Debug, Deserialize)]
pub struct Instance {
    /// Unique instance index (`Instance_Id_Type`, starts at 1).
    ///
    /// IDs may skip values when instances are freed; loaders insert dummy
    /// entries so [`InstanceId::index`](crate::InstanceId::index) still
    /// addresses the correct slot.
    pub id: InstanceId,

    /// Statement that created this scope, if any.
    ///
    /// One of: subprogram call, component/design instantiation, block, process,
    /// generate body, protected-object elaboration. `None` for root, packages,
    /// and verification units.
    #[serde(deserialize_with = "deserialize_optional_node_id")]
    pub stmt: Option<InstanceStatementNodeId>,

    /// Source construct for the scope (architecture, component, process, …).
    ///
    /// Defines which declaration chain and annotations own the object slots.
    #[serde(deserialize_with = "deserialize_optional_node_id")]
    pub source: Option<InstanceSourceNodeId>,

    /// Parent scope instance (`Up_Block`).
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub parent: Option<InstanceId>,

    /// Configuration bound to this instance, if any.
    ///
    /// Block configurations for architectures/generates; `None` for packages
    /// and process instances.
    #[serde(default, deserialize_with = "deserialize_optional_node_id")]
    pub config: Option<InstanceConfigurationNodeId>,

    /// Head of the verification-unit instance chain for this scope.
    ///
    /// Only the first extra instance is exported; further units are linked via
    /// `extra` on those instances in Ada (`Extra_Units` / `Extra_Link`).
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub extra: Option<InstanceId>,

    /// Block/entity header scope identity (`Get_Instance_Block_Ref`).
    ///
    /// For architecture instances this is typically the entity (`Get_Info_Scope`),
    /// not the architecture body.
    #[serde(deserialize_with = "deserialize_optional_node_id")]
    pub block: Option<InstanceBlockRefNodeId>,

    /// Uninstantiated package or subprogram specification scope.
    ///
    /// Set when elaborating an uninstantiated package body so that lookups from
    /// the body use the uninstantiated specification's annotations instead of
    /// the instance's `block` scope.
    ///
    /// ```vhdl
    /// package gen_pkg is
    ///   generic (type t);
    ///   procedure p (x : t);
    /// end;
    /// package body gen_pkg is
    ///   procedure p (x : t) is begin null; end;
    /// end;
    /// -- Instantiation elaborates a body instance with `uninst` → gen_pkg.
    /// ```
    #[serde(deserialize_with = "deserialize_optional_node_id")]
    pub uninst: Option<UninstantiatedScopeRefNodeId>,

    /// True when a subprogram has a signal formal associated by individual
    /// elements (`Get_Indiv_Signal_Assoc_Flag`).
    ///
    /// After a `wait`, simulation must refresh such formals from the actuals.
    ///
    /// ```vhdl
    /// procedure p (signal s : bit_vector) is
    /// begin
    ///   wait on clk;  -- s must be re-copied from the actual
    /// end;
    /// -- call: p((0 => a, 1 => b));  -- individual association → flag1
    /// ```
    #[serde(rename = "flag1")]
    pub has_individual_association_formals: bool,

    /// True when an ancestor instance has [`has_individual_association_formals`](Self::has_individual_association_formals) set.
    ///
    /// Propagates through ancestor instances so nested waits still refresh individual
    /// signal associations.
    #[serde(rename = "flag2")]
    pub has_ancestor_with_individual_association_formals: bool,

    /// Foreign-module binding id; `0` for ordinary VHDL.
    ///
    /// Foreign instances typically have a null block scope.
    pub foreign: i32,
}

impl Instance {
    /// Creates a placeholder for a missing instance table entry.
    ///
    /// Used when serialized instance IDs skip values so that
    /// [`InstanceId::index`] still addresses the corresponding slot.
    #[must_use]
    pub fn dummy(id: InstanceId) -> Self {
        Self {
            id,
            stmt: None,
            source: None,
            parent: None,
            config: None,
            extra: None,
            block: None,
            uninst: None,
            has_individual_association_formals: false,
            has_ancestor_with_individual_association_formals: false,
            foreign: 0,
        }
    }
}

impl IdIndex for Instance {
    fn id_index(&self) -> usize {
        self.id.index()
    }
}

/// Kind of entry stored in an instance object slot (`Obj_Kind`).
///
/// Slots are assigned during the annotation pass
/// (`Elab.Vhdl_Annotations`); elaboration fills them in declaration order.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjKind {
    /// Unused slot (not exported as an `object_slot` node).
    None,

    /// Constant, variable, signal, port, generic, file, etc. (`Valtyp`).
    Object,

    /// Subtype indication or interface type generic.
    Subtype,

    /// Interface subprogram generic.
    Subprg,

    /// Nested instance (component, package, generate child, process, …).
    Instance,
}

/// One occupied slot in an instance's object table.
///
/// Only slots with `obj_kind != none` are exported. Slot numbers are 1-based.
///
/// ```vhdl
/// architecture a of e is
///   constant c : integer := 1;   -- object slot
///   signal s : bit;              -- object slot (value → signal)
///   component comp is end;       -- may yield an instance slot at u:
/// begin
///   u: comp;                     -- object slot of kind instance
/// end;
/// ```
#[derive(Debug, Deserialize)]
pub struct ObjectSlot {
    /// Owning instance.
    pub instance: InstanceId,

    /// 1-based slot index within that instance (`Object_Slot_Type`).
    pub slot: u32,

    /// Declaration owning this slot, when found on the source declaration chain.
    ///
    /// For `obj_kind: subprg`, export overwrites this with the associated
    /// subprogram (`S_Decl`). Often `None` for ports, nested instances, and
    /// implicit objects not found on the declaration chain.
    #[serde(deserialize_with = "deserialize_optional_node_id")]
    pub decl: Option<DesignSlotDeclarationNodeId>,

    /// Runtime contents of the slot.
    #[serde(flatten)]
    pub kind: ObjectSlotKind,
}

impl IdIndex for ObjectSlot {
    fn id_index(&self) -> usize {
        self.instance.index()
    }
}

/// Runtime contents of an [`ObjectSlot`], tagged by `obj_kind`.
#[derive(Debug, Deserialize)]
#[serde(tag = "obj_kind", rename_all = "snake_case")]
pub enum ObjectSlotKind {
    /// Unused (should not appear in the export stream).
    None,

    /// Elaborated object with type and value (`Obj_Object` / `Valtyp`).
    ///
    /// Covers constants, variables, signals, ports, file objects, and most
    /// generics. For signals, [`value`](Self::Object::value) is typically a
    /// [`ValueKindData::Signal`].
    Object {
        #[serde(rename = "type")]
        typ: TypeId,
        value: ValueId,
    },

    /// Elaborated subtype indication (`Obj_Subtype`).
    ///
    /// Ordinary subtype objects have `def == None`. Interface type generics
    /// (`Create_Interface_Type`) store the actual type/subtype definition in
    /// `def`.
    ///
    /// ```vhdl
    /// package pkg is
    ///   generic (type t);
    /// end;
    /// package inst is new work.pkg generic map (t => integer);
    /// -- slot for `t` is subtype with def → integer
    /// ```
    Subtype {
        #[serde(rename = "type")]
        typ: TypeId,

        /// Actual type/subtype definition for an interface type generic
        /// (`Create_Interface_Type`); `None` for ordinary subtype objects.
        #[serde(deserialize_with = "deserialize_optional_node_id")]
        def: Option<TypeAndSubtypeDefinitionNodeId>,
    },

    /// Interface subprogram generic (`Obj_Subprg`); the bound subprogram is in
    /// [`ObjectSlot::decl`].
    Subprg,

    /// Nested instance pointer (`Obj_Instance`).
    ///
    /// Created for component/entity instantiations, package objects, generate
    /// children, and (after `Elab_Processes`) process sub-instances.
    Instance { target_instance: InstanceId },
}

/// Discriminant of a [`Signal`] entry (`Signal_Kind`).
///
/// User signals and ports are `User`. Implicit signals from attributes and
/// guarded blocks use the other variants; their `decl` points at the attribute
/// or guard declaration rather than a signal declaration.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalKind {
    /// Explicit signal or port declaration.
    User,

    /// Implicit signal `S'QUIET(T)` (IEEE 1076 §16.2.6 / §14.7.5).
    Quiet,

    /// Implicit signal `S'STABLE(T)`.
    Stable,

    /// Implicit signal `S'TRANSACTION`.
    Transaction,

    /// Implicit signal `S'DELAYED(T)`.
    Delayed,

    /// Implicit `GUARD` signal of a guarded block.
    ///
    /// ```vhdl
    /// b: block (en = '1')  -- creates implicit GUARD signal
    /// begin
    ///   q <= guarded d;
    /// end block;
    /// ```
    Guard,

    /// Placeholder allocated before `Gather_Signal` fills the entry.
    ///
    /// Should not appear in a completed export.
    None,
}

/// Simulation signal table entry (`Signal_Entry`).
///
/// Created in two phases: elaboration allocates a signal index and stores a
/// [`ValueKindData::Signal`] in an object slot; `Gather_Processes` then fills
/// this table (`Gather_Signal`). Source counts are finalized by
/// `Compute_Sources`.
///
/// Kind-specific fields: [`drivers`](Self::drivers) / [`disconnect`](Self::disconnect)
/// / [`nbr_sources`](Self::nbr_sources) apply to [`SignalKind::User`];
/// [`time`](Self::time) / [`pfx`](Self::pfx) apply to quiet/stable/delayed/
/// transaction.
#[derive(Debug, Deserialize)]
pub struct Signal {
    /// 1-based index into the signals table.
    pub id: SignalId,

    /// User vs implicit attribute/guard kind.
    pub signal_kind: SignalKind,

    /// Source declaration or implicit attribute for this signal.
    ///
    /// Implicit signals use `Attribute` with `kind` of `stable`, `quiet`,
    /// `transaction`, `delayed`, or `above`.
    #[serde(deserialize_with = "deserialize_optional_node_id")]
    pub decl: Option<SignalDeclNodeId>,

    /// Instance that owns this signal.
    pub instance: InstanceId,

    /// Elaborated type (after `Convert_Type_Width`, typically `wkind: sim`).
    #[serde(rename = "type")]
    pub typ: TypeId,

    /// Initial-value memory (default expression or type default).
    pub val_init: MemoryId,

    /// Current-value memory.
    ///
    /// For collapsed signals, this may point at the collapsed target's initial
    /// value rather than a separate buffer.
    pub val: MemoryId,

    /// Head of the sensitivity list of processes sensitive to this signal.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub sensitivity: Option<SensitivityId>,

    /// Head of the connection list involving this signal.
    ///
    /// Non-user signals may only appear as connection actuals, not formals.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub connect: Option<ConnectionId>,

    /// If set, this formal was collapsed onto that actual signal.
    ///
    /// Collapse happens when a full port association can share storage
    /// (`Get_Collapse_Signal_Flag`), e.g. `port map (o => x)` for an `out`
    /// port `o` and signal `x` of matching width at offset 0.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub collapsed_by: Option<SignalId>,

    /// Offset of this signal within [`collapsed_by`](Self::collapsed_by).
    pub collapsed_offs: ValueOffsets,

    /// Whether the declaration needs `'ACTIVE` tracking (`Get_Has_Active_Flag`).
    ///
    /// Propagated onto collapsed targets during `Compute_Sources`.
    pub has_active: bool,

    /// Head of the driver list (`user` only).
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub drivers: Option<DriverId>,

    /// Head of the disconnection-specification list (`user` only).
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub disconnect: Option<DisconnectId>,

    /// Per-scalar-element source counts (`user` only).
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub nbr_sources: Option<NbrSourcesId>,

    /// Time parameter *T* of `S'QUIET(T)`, `S'STABLE(T)`, or `S'DELAYED(T)`.
    ///
    /// Zero for `S'TRANSACTION`. Absent for user/guard/above.
    #[serde(default)]
    pub time: Option<i64>,

    /// Prefix signal *S* for quiet/stable/delayed/transaction attributes.
    #[serde(default)]
    pub pfx: Option<SubSignal>,
}

impl IdIndex for Signal {
    fn id_index(&self) -> usize {
        self.id.index()
    }
}

/// Explicit or implicit process in the elaborated design (`Proc_Record_Type`).
///
/// Beyond `process` statements, GHDL registers concurrent signal assignments,
/// concurrent assertions, concurrent procedure calls, PSL directives, and
/// non-static port associations (as implicit processes) during
/// `Gather_Processes`. True process statements get a dedicated sub-instance
/// later in `Elab_Processes`.
///
/// ```vhdl
/// -- Each of these yields a process entry:
/// y <= a and b;  -- concurrent assignment
/// assert en = '1';  -- concurrent assertion
/// p: process (clk) begin ... end;  -- explicit process
/// ```
#[derive(Debug, Deserialize)]
pub struct Process {
    /// 1-based index into the processes table.
    pub id: ProcessId,

    /// Process or concurrent statement that owns this process entry.
    ///
    /// PSL assert/assume/cover/endpoint directives may also appear; those IIR
    /// kinds are not yet included in [`ProcessStmtNodeId`].
    #[serde(deserialize_with = "deserialize_optional_node_id")]
    pub proc: Option<ProcessStmtNodeId>,

    /// Scope instance.
    ///
    /// After `Elab_Processes`, may be the process's own sub-instance rather
    /// than the enclosing architecture/block.
    pub instance: InstanceId,

    /// Head of the list of signals this process drives (`prev_proc` chain).
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub drivers: Option<DriverId>,

    /// Head of the sensitivity list (`prev_proc` chain).
    ///
    /// Sensitized processes and concurrent statements contribute here. A plain
    /// `process` without a sensitivity list has no exported sensitivity; waits
    /// are handled at runtime.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub sensitivity: Option<SensitivityId>,
}

impl Process {
    /// Iterates drivers of this process (follows [`Driver::prev_proc`] from the head).
    #[must_use]
    pub fn drivers<'design>(&self, design: &'design Design) -> Drivers<'design> {
        Drivers {
            design,
            next: self.drivers,
        }
    }

    /// Iterates sensitivity entries of this process (follows [`Sensitivity::prev_proc`]).
    #[must_use]
    pub fn sensitivity_list<'design>(&self, design: &'design Design) -> SensitivityList<'design> {
        SensitivityList {
            design,
            next: self.sensitivity,
        }
    }
}

impl IdIndex for Process {
    fn id_index(&self) -> usize {
        self.id.index()
    }
}

/// One process driving (a part of) a signal (`Driver_Entry`).
///
/// Drivers form two linked lists: [`prev_sig`](Self::prev_sig) chains all
/// drivers of the same signal (head at [`Signal::drivers`]);
/// [`prev_proc`](Self::prev_proc) chains all signals driven by the same
/// process (head at [`Process::drivers`]).
///
/// ```vhdl
/// p: process begin
///   y <= a and b;  -- driver on y
///   z <= c;        -- second driver entry, same proc
/// end process;
/// ```
#[derive(Debug, Deserialize)]
pub struct Driver {
    /// 1-based index into the drivers table.
    pub id: DriverId,

    /// Signal (or subelement) being driven.
    pub sig: SubSignal,

    /// Driving process.
    pub proc: ProcessId,

    /// Previous driver of the same signal.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub prev_sig: Option<DriverId>,

    /// Previous signal driven by the same process.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub prev_proc: Option<DriverId>,
}

impl IdIndex for Driver {
    fn id_index(&self) -> usize {
        self.id.index()
    }
}

/// Iterator over a process's drivers (follows [`Driver::prev_proc`]).
#[derive(Clone)]
pub struct Drivers<'design> {
    design: &'design Design,
    next: Option<DriverId>,
}

impl<'design> Iterator for Drivers<'design> {
    type Item = &'design Driver;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next?;
        let driver = self.design.drivers.get(next.index())?;
        self.next = driver.prev_proc;
        Some(driver)
    }
}

/// One sensitivity edge: process resumes on an event on (part of) a signal.
///
/// Same record shape as [`Driver`] (`Sensitivity_Entry` is a subtype of
/// `Driver_Entry` in Ada). Built from explicit sensitivity lists, concurrent
/// statement analysis, and PSL clocks.
#[derive(Debug, Deserialize)]
pub struct Sensitivity {
    /// 1-based index into the sensitivity table.
    pub id: SensitivityId,

    /// Signal (or subelement) that wakes the process.
    pub sig: SubSignal,

    /// Sensitive process.
    pub proc: ProcessId,

    /// Previous sensitivity entry for the same signal.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub prev_sig: Option<SensitivityId>,

    /// Previous sensitivity entry for the same process.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub prev_proc: Option<SensitivityId>,
}

impl IdIndex for Sensitivity {
    fn id_index(&self) -> usize {
        self.id.index()
    }
}

/// Iterator over a process's sensitivity list (follows [`Sensitivity::prev_proc`]).
#[derive(Clone)]
pub struct SensitivityList<'design> {
    design: &'design Design,
    next: Option<SensitivityId>,
}

impl<'design> Iterator for SensitivityList<'design> {
    type Item = &'design Sensitivity;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next?;
        let sensitivity = self.design.sensitivities.get(next.index())?;
        self.next = sensitivity.prev_proc;
        Some(sensitivity)
    }
}

/// Port (or similar) association between formal and actual sub-signals
/// (`Connect_Entry`).
///
/// Built by `Gather_Connections` for each non-open, non-individual association
/// in a port map. Formal and actual each have a linked list of connections
/// (`formal_link` / `actual_link`); signal heads are at [`Signal::connect`].
///
/// Open associations (`open`) and associations by individual elements do
/// **not** create connection entries.
///
/// ```vhdl
/// u: entity work.child
///   port map (
///     a => x,           -- connection; may collapse if widths match
///     b => open,        -- no connection entry
///     c(0) => y0,       -- individual → no connection entry here
///     c(1) => y1
///   );
/// ```
///
/// When [`collapsed`](Self::collapsed) is true, the formal signal's
/// [`Signal::collapsed_by`] points at the actual so both share storage.
#[derive(Debug, Deserialize)]
pub struct Connection {
    /// 1-based index into the connections table.
    pub id: ConnectionId,

    /// Formal endpoint (port or interface signal).
    pub formal: SubSignal,

    /// Actual endpoint (local signal, or open with `base == None`).
    pub actual: SubSignal,

    /// Next connection sharing the same formal signal.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub formal_link: Option<ConnectionId>,

    /// Next connection sharing the same actual signal.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub actual_link: Option<ConnectionId>,

    /// True when the formal is elided onto (a slice of) the actual.
    pub collapsed: bool,

    /// Association element that created this connection.
    ///
    /// Open and by-individual associations do not create connection entries.
    #[serde(deserialize_with = "deserialize_optional_node_id")]
    pub assoc: Option<ConnectionAssociationNodeId>,

    /// Instance in whose context the association was written.
    pub assoc_inst: InstanceId,
}

impl IdIndex for Connection {
    fn id_index(&self) -> usize {
        self.id.index()
    }
}

/// Disconnection specification for a guarded signal (`Disconnect_Entry`).
///
/// Corresponds to a `disconnect` declaration (IEEE 1076 §5.2.1.2 / §14.4.3.4).
/// Linked via [`prev`](Self::prev) with the head at [`Signal::disconnect`].
///
/// ```vhdl
/// signal q : std_logic bus;
/// disconnect q : std_logic after 5 ns;
/// ```
#[derive(Debug, Deserialize)]
pub struct Disconnect {
    /// 1-based index into the disconnect table.
    pub id: DisconnectId,

    /// Affected signal (or subelement).
    pub sig: SubSignal,

    /// Previous disconnect entry for the same signal.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub prev: Option<DisconnectId>,

    /// Disconnection time (`Std_Time` as femtoseconds / GHDL time units).
    pub val: i64,
}

impl IdIndex for Disconnect {
    fn id_index(&self) -> usize {
        self.id.index()
    }
}

/// Elaborated type kind (`Type_Kind` in `Elab.Vhdl_Objtypes`).
///
/// Distinguishes scalar nets, bounded vs unbounded composites, and
/// non-synthesizable types. Multidimensional arrays are represented as a chain
/// of array types linked by [`TypeNode::element_type`], with
/// [`TypeNode::is_last`] marking the innermost dimension.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeKind {
    /// Predefined `BIT` (1-bit net).
    Bit,

    /// Multi-value logic (`STD_LOGIC` / `STD_ULOGIC` family).
    Logic,

    /// Integer or enumeration subtype.
    Discrete,

    /// Floating-point subtype.
    Float,

    /// Slice of a vector with known width but dynamic bounds.
    ///
    /// ```vhdl
    /// -- s(v'range) where v is an unconstrained formal → Type_Slice
    /// ```
    Slice,

    /// Bounded 1-D array of a net type (synthesizable vector).
    ///
    /// ```vhdl
    /// signal u : unsigned(7 downto 0);  -- Type_Vector of logic elements
    /// ```
    Vector,

    /// Unbounded 1-D vector (`array (natural range <>) of bit`, etc.).
    UnboundedVector,

    /// Fully bounded (possibly multi-dimensional) array.
    Array,

    /// Array with known indexes but unbounded element type.
    ///
    /// ```vhdl
    /// type t is array (0 to 3) of bit_vector;  -- element unconstrained
    /// ```
    ArrayUnbounded,

    /// Array without static index bounds (and possibly unbounded elements).
    UnboundedArray,

    /// Record with at least one unbounded element.
    UnboundedRecord,

    /// Fully bounded record.
    Record,

    /// Access type.
    Access,

    /// File type (non-synthesizable; `w` defaults to 32).
    File,

    /// Protected type (non-synthesizable).
    Protected,
}

/// Meaning of [`TypeNode::w`] (`Wkind_Type`).
///
/// Before signal export, `Convert_Type_Width` rewrites types used by simulation
/// to [`Sim`](Self::Sim) so `w` counts scalar elements.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Wkind {
    /// Width not defined.
    Undef,

    /// `w` is the number of synthesis nets / bits.
    Net,

    /// `w` is the number of scalar elements (simulation).
    Sim,
}

/// Interned elaborated type (`Type_Type`).
///
/// Types are interned by pointer address in the export adapter; identical
/// [`TypeId`] values mean the same GHDL `Type_Acc`. Fields after the common
/// header are kind-dependent (see [`type_kind`](Self::type_kind)).
///
/// Common header: alignment, byte size [`sz`](Self::sz), and width [`w`](Self::w).
#[derive(Debug, Deserialize)]
pub struct TypeNode {
    /// Interned type id.
    pub id: TypeId,

    /// Structural kind.
    pub type_kind: TypeKind,

    /// Interpretation of [`w`](Self::w).
    pub wkind: Wkind,

    /// Power-of-two byte alignment (`Palign_Type`, 0..=3 → 1/2/4/8 bytes).
    pub align: u32,

    /// Memory size in bytes.
    pub sz: i64,

    /// Width: nets/bits ([`Wkind::Net`]) or scalar elements ([`Wkind::Sim`]).
    ///
    /// May be zero for null arrays or single-value discrete subtypes
    /// (`range 0 to 0`).
    pub w: u32,

    /// Left bound of a scalar or array dimension.
    #[serde(default)]
    pub left: Option<ScalarBound>,

    /// Right bound of a scalar or array dimension.
    #[serde(default)]
    pub right: Option<ScalarBound>,

    /// Direction of a scalar or array range (`to` / `downto`).
    #[serde(default)]
    pub dir: Option<Direction>,

    /// Signed vs unsigned net representation for discrete types.
    #[serde(default)]
    pub is_signed: Option<bool>,

    /// Base type for slices (`Slice_Base`) or records (`Rec_Base`).
    ///
    /// Record base types provide a compatible layout across subtypes.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub base_type: Option<TypeId>,

    /// Length of a vector/array dimension or slice (`Abound.Len` / `Slice_Len`).
    #[serde(default)]
    pub len: Option<u32>,

    /// Element type of an array, vector, or slice.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub element_type: Option<TypeId>,

    /// Index subtype of an unbounded array/vector.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub index_type: Option<TypeId>,

    /// True on the last (innermost) array/vector dimension.
    #[serde(default)]
    pub is_last: Option<bool>,

    /// Record elements in declaration order (first field in LSBs of the net).
    #[serde(default)]
    pub fields: Option<Vec<RecordField>>,

    /// Designated type of an access type (`Acc_Acc`).
    ///
    /// May be absent for incomplete access types until completion.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub designated_type: Option<TypeId>,

    /// Memory size to store an access value's designated object.
    #[serde(default)]
    pub acc_type_sz: Option<i64>,

    /// Memory size for bounds when the designated type can be a signal.
    #[serde(default)]
    pub acc_bnd_sz: Option<i64>,

    /// Element type of a file type.
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub file_type: Option<TypeId>,

    /// GHDL file-type signature string.
    #[serde(default)]
    pub signature: Option<String>,
}

impl IdIndex for TypeNode {
    fn id_index(&self) -> usize {
        self.id.index()
    }
}

/// Scalar range endpoint: discrete as `i64`, float as `f64`.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ScalarBound {
    /// Floating-point bound (`Frange`).
    Float(#[serde(deserialize_with = "deserialize_f64")] f64),

    /// Integer/enumeration bound (`Drange` / `Abound`).
    Discrete(i64),
}

/// One element of an elaborated record type (`Rec_El_Type`).
#[derive(Debug, Deserialize)]
pub struct RecordField {
    /// Element type.
    #[serde(rename = "type")]
    pub typ: TypeId,

    /// Offset of this element within the parent record.
    pub offs: ValueOffsets,

    /// Element declaration in the AST, if available.
    #[serde(deserialize_with = "deserialize_optional_node_id")]
    pub decl: Option<RecordFieldDeclNodeId>,
}

impl IdIndex for RecordField {
    fn id_index(&self) -> usize {
        self.typ.index()
    }
}

/// Interned value node (`Value_Type`).
///
/// Values decompose objects and signals for synthesis and simulation. A GHDL
/// `Valtyp` is the pair `(type, value)`.
#[derive(Debug, Deserialize)]
pub struct Value {
    /// Interned value id.
    pub id: ValueId,

    /// Kind-specific payload.
    #[serde(flatten)]
    pub kind: ValueKindData,
}

impl IdIndex for Value {
    fn id_index(&self) -> usize {
        self.id.index()
    }
}

/// Kind-specific payload of a [`Value`] (`Value_Kind`).
#[derive(Debug, Deserialize)]
#[serde(tag = "val_kind", rename_all = "snake_case")]
pub enum ValueKindData {
    /// User signal: index into the signals table.
    ///
    /// During elaboration the slot is preallocated (`Signal_None`);
    /// `Gather_Signal` fills the entry.
    Signal { signal: SignalId },

    /// Raw constant or runtime data blob.
    Memory { memory: MemoryId },

    /// Open file object (`Ghdl_File_Index`).
    File { file: u32 },

    /// Named constant wrapping another value (`Create_Value_Const`).
    ///
    /// `loc` is the constant declaration.
    Const {
        value: ValueId,

        /// Source declaration of the named constant (`Create_Value_Const`).
        #[serde(deserialize_with = "deserialize_optional_node_id")]
        loc: Option<NodeId<ConstantDeclaration>>,
    },

    /// Static view of another value with a compatible type (slice, field, alias).
    ///
    /// ```vhdl
    /// alias a : bit is s(3);     -- alias into signal/value at offs
    /// signal r : rec_t;
    /// -- r.field is an alias with offs into r
    /// ```
    Alias {
        obj: ValueId,
        #[serde(rename = "type")]
        typ: TypeId,
        offs: ValueOffsets,
    },

    /// Dynamic index into a prefix value (used in associations).
    ///
    /// `poff` is the fixed offset from `obj`; `ptype` is the prefix type after
    /// that offset; `voff` is the variable (computed) part; `eoff` is an
    /// additional fixed element offset.
    ///
    /// ```vhdl
    /// -- port map (f => v(i)) with non-static i → dyn_alias
    /// ```
    DynAlias {
        obj: ValueId,
        poff: u32,
        ptype: TypeId,
        voff: u32,
        eoff: u32,
    },

    /// Bundle for individual signal associations in simulation.
    ///
    /// `sigs` / `vals` are parallel memory arrays tracking per-element actual
    /// signals and values (`Hook_Create_Value_For_Signal_Individual_Assocs`).
    SigVal {
        #[serde(default, deserialize_with = "deserialize_optional_id")]
        sigs: Option<MemoryId>,
        #[serde(default, deserialize_with = "deserialize_optional_id")]
        vals: Option<MemoryId>,
    },
}

/// Interned raw memory blob (Base64-encoded in JSON).
///
/// Backing store for constant values, signal `val` / `val_init`, and similar
/// buffers. `size` is the byte length; `data` is decoded from standard Base64.
#[derive(Debug, Deserialize)]
pub struct Memory {
    /// Interned memory id.
    pub id: MemoryId,

    /// Declared byte length of the blob.
    pub size: u32,

    /// Decoded contents.
    #[serde(deserialize_with = "deserialize_base64")]
    pub data: Vec<u8>,
}

impl IdIndex for Memory {
    fn id_index(&self) -> usize {
        self.id.index()
    }
}

/// Per-scalar-element source-count array for a user signal (`Nbr_Sources_Array`).
///
/// Length equals the signal type's [`TypeNode::w`] (one entry per scalar
/// subelement). Used to detect multiple drivers and to size resolution.
#[derive(Debug, Deserialize)]
pub struct NbrSources {
    /// Interned nbr-sources id.
    pub id: NbrSourcesId,

    /// One entry per scalar element (`0 .. typ.w - 1`).
    pub entries: Vec<NbrSourcesEntry>,
}

impl IdIndex for NbrSources {
    fn id_index(&self) -> usize {
        self.id.index()
    }
}

/// Source counts for one scalar signal element (`Nbr_Sources_Type`).
///
/// `total` is finalized in `Compute_Sources` and includes sources contributed
/// through collapsed signals. Resolved subtypes are pre-marked with
/// `total == 1`. Undriven `out` ports receive a default source.
#[derive(Debug, Deserialize)]
pub struct NbrSourcesEntry {
    /// Distinct processes driving this element.
    pub nbr_drivers: u32,

    /// Sources from OUT/INOUT/BUFFER/LINKAGE port connections.
    pub nbr_conns: u32,

    /// Final source count after collapse propagation.
    pub total: u32,
}

/// Dual offset into net (scalar-element) and memory (byte) layouts
/// (`Value_Offsets`).
///
/// Used for subsignals, aliases, record fields, and collapse offsets.
/// `net_off` indexes the scalar-element space used by drivers and
/// [`NbrSources`]; `mem_off` is the byte offset in the memory representation.
#[derive(Debug, Deserialize)]
pub struct ValueOffsets {
    /// Index into scalar-element / net space.
    pub net_off: u32,

    /// Byte offset in memory layout.
    pub mem_off: i64,
}

/// Signal or subelement thereof (`Sub_Signal_Type`).
///
/// Identifies a whole signal (`offs` zero, `typ` often omitted) or a slice /
/// record field / array element within a composite signal. Used by drivers,
/// sensitivity, connections, and disconnect entries.
///
/// ```vhdl
/// y(3 downto 0) <= x;  -- driver SubSignal: base=y, offs into the slice
/// ```
#[derive(Debug, Deserialize)]
pub struct SubSignal {
    /// Base signal; `None` when the endpoint is open / unbound (`No_Signal_Index`).
    #[serde(deserialize_with = "deserialize_optional_id")]
    pub base: Option<SignalId>,

    /// Offset within the base signal's value.
    pub offs: ValueOffsets,

    /// Subelement type; omitted when identical to the base signal's type.
    #[serde(default, rename = "type", deserialize_with = "deserialize_optional_id")]
    pub typ: Option<TypeId>,
}

/// Deserializes a standard Base64 string into raw bytes.
fn deserialize_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let encoded = <&str>::deserialize(deserializer)?;
    decode_base64(encoded).map_err(D::Error::custom)
}

/// Decodes standard Base64 (`A-Za-z0-9+/` with `=` padding) into bytes.
fn decode_base64(input: &str) -> Result<Vec<u8>, &'static str> {
    if !input.len().is_multiple_of(4) {
        return Err("invalid base64 length");
    }

    let mut out = Vec::with_capacity(input.len() / 4 * 3);
    for chunk in input.as_bytes().chunks_exact(4) {
        let b0 = decode_base64_digit(chunk[0])?;
        let b1 = decode_base64_digit(chunk[1])?;
        out.push((b0 << 2) | (b1 >> 4));

        if chunk[2] == b'=' {
            if chunk[3] != b'=' {
                return Err("invalid base64 padding");
            }
            break;
        }
        let b2 = decode_base64_digit(chunk[2])?;
        out.push(((b1 & 0xf) << 4) | (b2 >> 2));

        if chunk[3] == b'=' {
            break;
        }
        let b3 = decode_base64_digit(chunk[3])?;
        out.push(((b2 & 0x3) << 6) | b3);
    }
    Ok(out)
}

/// Maps one Base64 alphabet character to its 6-bit value.
fn decode_base64_digit(byte: u8) -> Result<u8, &'static str> {
    match byte {
        b'A'..=b'Z' => Ok(byte - b'A'),
        b'a'..=b'z' => Ok(byte - b'a' + 26),
        b'0'..=b'9' => Ok(byte - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        _ => Err("invalid base64 character"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Appends standard Base64 encoding of `bytes` to `buffer`.
    fn append_base64(buffer: &mut Vec<u8>, bytes: &[u8]) {
        const TABLE: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut chunks = bytes.chunks_exact(3);
        for chunk in chunks.by_ref() {
            let n = (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8) | u32::from(chunk[2]);
            buffer.push(TABLE[((n >> 18) & 0x3f) as usize]);
            buffer.push(TABLE[((n >> 12) & 0x3f) as usize]);
            buffer.push(TABLE[((n >> 6) & 0x3f) as usize]);
            buffer.push(TABLE[(n & 0x3f) as usize]);
        }
        let rem = chunks.remainder();
        match rem.len() {
            1 => {
                let n = u32::from(rem[0]) << 16;
                buffer.push(TABLE[((n >> 18) & 0x3f) as usize]);
                buffer.push(TABLE[((n >> 12) & 0x3f) as usize]);
                buffer.push(b'=');
                buffer.push(b'=');
            },
            2 => {
                let n = (u32::from(rem[0]) << 16) | (u32::from(rem[1]) << 8);
                buffer.push(TABLE[((n >> 18) & 0x3f) as usize]);
                buffer.push(TABLE[((n >> 12) & 0x3f) as usize]);
                buffer.push(TABLE[((n >> 6) & 0x3f) as usize]);
                buffer.push(b'=');
            },
            _ => {},
        }
    }

    fn encode_base64(bytes: &[u8]) -> String {
        let mut buffer = Vec::new();
        append_base64(&mut buffer, bytes);
        String::from_utf8(buffer).expect("base64 encoding should be valid UTF-8")
    }

    /// RFC 4648 §10 test vectors.
    const RFC4648_VECTORS: &[(&[u8], &str)] = &[
        (b"", ""),
        (b"f", "Zg=="),
        (b"fo", "Zm8="),
        (b"foo", "Zm9v"),
        (b"foob", "Zm9vYg=="),
        (b"fooba", "Zm9vYmE="),
        (b"foobar", "Zm9vYmFy"),
    ];

    #[test]
    fn encode_rfc4648_vectors() {
        for &(plain, encoded) in RFC4648_VECTORS {
            assert_eq!(encode_base64(plain), encoded, "encode {plain:?}");
        }
    }

    #[test]
    fn decode_rfc4648_vectors() {
        for &(plain, encoded) in RFC4648_VECTORS {
            assert_eq!(
                decode_base64(encoded).expect("valid Base64-encoded string"),
                plain,
                "decode {encoded}"
            );
        }
    }

    #[test]
    fn round_trip_all_byte_lengths() {
        let mut bytes = Vec::new();
        for len in 0..=64 {
            bytes.resize(len, 0);
            for (i, byte) in bytes.iter_mut().enumerate() {
                let value = i * 37 + len;
                *byte = value.to_le_bytes()[0];
            }
            let encoded = encode_base64(&bytes);
            assert_eq!(
                decode_base64(&encoded).expect("valid Base64-encoded string"),
                bytes,
                "len={len}"
            );
        }
    }

    #[test]
    fn round_trip_full_alphabet_payload() {
        let bytes: Vec<u8> = (0_u8..=255).collect();
        let encoded = encode_base64(&bytes);
        assert_eq!(
            decode_base64(&encoded).expect("valid Base64-encoded string"),
            bytes
        );
    }

    #[test]
    fn decode_rejects_invalid_length() {
        assert_eq!(decode_base64("A"), Err("invalid base64 length"));
        assert_eq!(decode_base64("AB"), Err("invalid base64 length"));
        assert_eq!(decode_base64("ABC"), Err("invalid base64 length"));
    }

    #[test]
    fn decode_rejects_invalid_character() {
        assert_eq!(decode_base64("!!!!"), Err("invalid base64 character"));
        assert_eq!(decode_base64("Zm9vYmF!"), Err("invalid base64 character"));
    }

    #[test]
    fn decode_rejects_invalid_padding() {
        assert_eq!(decode_base64("Zg=A"), Err("invalid base64 padding"));
    }

    #[test]
    fn deserialize_memory_data_field() {
        let memory: Memory =
            serde_json::from_str(r#"{"id":1,"size":3,"data":"Zm9v"}"#).expect("valid JSON");
        assert_eq!(memory.id.to_raw().get(), 1);
        assert_eq!(memory.size, 3);
        assert_eq!(memory.data, b"foo");
    }
}
