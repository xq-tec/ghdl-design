//! Subset ID types for AST nodes referenced from the elaborated design.

ghdl_ast::subset_declaration!(InstanceStatement InstanceStatementOwned InstanceStatementNodeId {
    ComponentInstantiation(ComponentInstantiationStatement),
    Block(BlockStatement),
    Process(ProcessStatement),
    SensitizedProcess(SensitizedProcessStatement),
    GenerateStatementBody(GenerateStatementBody),
    ForGenerate(ForGenerateStatement),
    IfGenerate(IfGenerateStatement),
    CaseGenerate(CaseGenerateStatement),
    ProcedureCall(ProcedureCallStatement),
    VariableDeclaration(VariableDeclaration),
});

ghdl_ast::subset_declaration!(InstanceSource InstanceSourceOwned InstanceSourceNodeId {
    ArchitectureBody(ArchitectureBody),
    EntityDeclaration(EntityDeclaration),
    ComponentDeclaration(ComponentDeclaration),
    PackageDeclaration(PackageDeclaration),
    PackageInstantiationDeclaration(PackageInstantiationDeclaration),
    ConfigurationDeclaration(ConfigurationDeclaration),
    BlockStatement(BlockStatement),
    GenerateStatementBody(GenerateStatementBody),
    ForGenerate(ForGenerateStatement),
    IfGenerate(IfGenerateStatement),
    CaseGenerate(CaseGenerateStatement),
    Process(ProcessStatement),
    SensitizedProcess(SensitizedProcessStatement),
    FunctionBody(FunctionBody),
    ProcedureBody(ProcedureBody),
});

ghdl_ast::subset_declaration!(InstanceConfiguration InstanceConfigurationOwned InstanceConfigurationNodeId {
    BlockConfiguration(BlockConfiguration),
    ComponentConfiguration(ComponentConfiguration),
});

ghdl_ast::subset_declaration!(InstanceBlockRef InstanceBlockRefOwned InstanceBlockRefNodeId {
    EntityDeclaration(EntityDeclaration),
    ArchitectureBody(ArchitectureBody),
    ComponentDeclaration(ComponentDeclaration),
    PackageDeclaration(PackageDeclaration),
    PackageInstantiationDeclaration(PackageInstantiationDeclaration),
    ConfigurationDeclaration(ConfigurationDeclaration),
    BlockStatement(BlockStatement),
    GenerateStatementBody(GenerateStatementBody),
    ForGenerate(ForGenerateStatement),
    IfGenerate(IfGenerateStatement),
    CaseGenerate(CaseGenerateStatement),
    Process(ProcessStatement),
    SensitizedProcess(SensitizedProcessStatement),
    ComponentInstantiation(ComponentInstantiationStatement),
    FunctionDeclaration(FunctionDeclaration),
    ProcedureDeclaration(ProcedureDeclaration),
    ForeignModule(ForeignModule),
});

ghdl_ast::subset_declaration!(UninstantiatedScopeRef UninstantiatedScopeRefOwned UninstantiatedScopeRefNodeId {
    PackageDeclaration(PackageDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    ProcedureDeclaration(ProcedureDeclaration),
});

ghdl_ast::subset_declaration!(DesignSlotDeclaration DesignSlotDeclarationOwned DesignSlotDeclarationNodeId {
    SignalDeclaration(SignalDeclaration),
    InterfaceSignalDeclaration(InterfaceSignalDeclaration),
    ConstantDeclaration(ConstantDeclaration),
    InterfaceConstantDeclaration(InterfaceConstantDeclaration),
    VariableDeclaration(VariableDeclaration),
    InterfaceVariableDeclaration(InterfaceVariableDeclaration),
    FileDeclaration(FileDeclaration),
    InterfaceFileDeclaration(InterfaceFileDeclaration),
    InterfaceViewDeclaration(InterfaceViewDeclaration),
    IteratorDeclaration(IteratorDeclaration),
    ObjectAliasDeclaration(ObjectAliasDeclaration),
    AttributeValue(AttributeValue),
    Attribute(Attribute),
    PackageInstantiationDeclaration(PackageInstantiationDeclaration),
    InterfacePackageDeclaration(InterfacePackageDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    ProcedureDeclaration(ProcedureDeclaration),
    FunctionInstantiationDeclaration(FunctionInstantiationDeclaration),
    ProcedureInstantiationDeclaration(ProcedureInstantiationDeclaration),
    InterfaceFunctionDeclaration(InterfaceFunctionDeclaration),
    InterfaceProcedureDeclaration(InterfaceProcedureDeclaration),
    ComponentInstantiationStatement(ComponentInstantiationStatement),
    SuspendStateDeclaration(SuspendStateDeclaration),
    GuardSignalDeclaration(GuardSignalDeclaration),
});

ghdl_ast::subset_declaration!(SignalDecl SignalDeclOwned SignalDeclNodeId {
    SignalDeclaration(SignalDeclaration),
    InterfaceSignalDeclaration(InterfaceSignalDeclaration),
    InterfaceViewDeclaration(InterfaceViewDeclaration),
    GuardSignalDeclaration(GuardSignalDeclaration),
    /// Implicit signals (`'stable`, `'quiet`, …) exported as consolidated `Attribute` nodes.
    Attribute(Attribute),
});

ghdl_ast::subset_declaration!(ProcessStmt ProcessStmtOwned ProcessStmtNodeId {
    Process(ProcessStatement),
    SensitizedProcess(SensitizedProcessStatement),
    ConcurrentSimpleSignalAssignment(ConcurrentSimpleSignalAssignment),
    ConcurrentConditionalSignalAssignment(ConcurrentConditionalSignalAssignment),
    ConcurrentSelectedSignalAssignment(ConcurrentSelectedSignalAssignment),
    ConcurrentAssertionStatement(ConcurrentAssertionStatement),
    ConcurrentProcedureCallStatement(ConcurrentProcedureCallStatement),
    /// Implicit driver process for a non-static port actual.
    AssociationElementByExpression(AssociationElementByExpression),
});

ghdl_ast::subset_declaration!(ConnectionAssociation ConnectionAssociationOwned ConnectionAssociationNodeId {
    ByName(AssociationElementByName),
    ByExpression(AssociationElementByExpression),
});

ghdl_ast::subset_declaration!(RecordFieldDecl RecordFieldDeclOwned RecordFieldDeclNodeId {
    ElementDeclaration(ElementDeclaration),
    RecordElementConstraint(RecordElementConstraint),
});
