mod control;

pub use control::construct_assembly as construct_control_flow_assembly;
pub use control::construct_lir as construct_control_flow_lir;
pub use control::destruct_assembly as destruct_control_flow_assembly;
pub use control::destruct_lir as destruct_control_flow_lir;
pub use control::Control;
pub use control::Edge;
